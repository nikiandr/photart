use std::fs::File;
use std::io::{self, BufReader, Cursor, Read};
use std::path::Path;
use std::sync::Mutex;

use exif::{Exif, In, Reader, Tag, Value};
use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, ImageFormat, ImageReader};
use serde::Serialize;

mod heic;
mod tiff;

pub const MAX_THUMBNAIL_DIMENSION: u32 = 640;
pub const THUMBNAIL_JPEG_QUALITY: u8 = 82;
const MAX_DETAIL_DIMENSION: u32 = 2560;
const DETAIL_JPEG_QUALITY: u8 = 90;
const MAX_FILE_SIZE_BYTES: u64 = 100 * 1024 * 1024;
const MAX_MEGAPIXELS: u64 = 100_000_000;
static DERIVATIVE_DECODE: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SupportedFormat {
    Raster(ImageFormat),
    Heic,
    Tiff,
}

impl SupportedFormat {
    fn from_path(path: &Path) -> Option<Self> {
        match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
            "jpg" | "jpeg" => Some(Self::Raster(ImageFormat::Jpeg)),
            "png" => Some(Self::Raster(ImageFormat::Png)),
            "webp" => Some(Self::Raster(ImageFormat::WebP)),
            "heic" | "heif" => Some(Self::Heic),
            "tif" | "tiff" => Some(Self::Tiff),
            _ => None,
        }
    }

    fn needs_detail_jpeg(self) -> bool {
        matches!(self, Self::Heic | Self::Tiff)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedPhotograph {
    pub width: u32,
    pub height: u32,
    pub thumbnail_jpeg: Vec<u8>,
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CaptureDetails {
    camera: Option<String>,
    lens: Option<String>,
    captured_at: Option<String>,
    aperture: Option<String>,
    shutter_speed: Option<String>,
    iso: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageProcessingError {
    UnsupportedFormat,
    DecodeError,
    TooLarge,
    PreviewError,
}

pub fn is_supported_extension(path: &Path) -> bool {
    SupportedFormat::from_path(path).is_some()
}

pub(crate) fn uses_detail_cache(path: &Path) -> bool {
    SupportedFormat::from_path(path).is_some_and(SupportedFormat::needs_detail_jpeg)
}

pub(crate) fn read_source(path: &Path) -> io::Result<Vec<u8>> {
    let file = File::open(path)?;
    if file.metadata()?.len() > MAX_FILE_SIZE_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Source exceeds 100 MiB",
        ));
    }
    let mut bytes = Vec::new();
    file.take(MAX_FILE_SIZE_BYTES + 1).read_to_end(&mut bytes)?;
    Ok(bytes)
}

pub fn prepare_photograph(
    path: &Path,
    bytes: &[u8],
) -> Result<PreparedPhotograph, ImageProcessingError> {
    let (_decode_slot, image) = decode_oriented(path, bytes)?;
    Ok(PreparedPhotograph {
        width: image.width(),
        height: image.height(),
        thumbnail_jpeg: render_jpeg(&image, MAX_THUMBNAIL_DIMENSION, THUMBNAIL_JPEG_QUALITY)?,
    })
}

pub(crate) fn prepare_detail_jpeg(
    path: &Path,
    bytes: &[u8],
) -> Result<Vec<u8>, ImageProcessingError> {
    if !uses_detail_cache(path) {
        return Err(ImageProcessingError::UnsupportedFormat);
    }
    let (_decode_slot, image) = decode_oriented(path, bytes)?;
    render_jpeg(&image, MAX_DETAIL_DIMENSION, DETAIL_JPEG_QUALITY)
}

fn decode_oriented(
    path: &Path,
    bytes: &[u8],
) -> Result<(Option<std::sync::MutexGuard<'static, ()>>, DynamicImage), ImageProcessingError> {
    if bytes.len() as u64 > MAX_FILE_SIZE_BYTES {
        return Err(ImageProcessingError::TooLarge);
    }

    let format = SupportedFormat::from_path(path).ok_or(ImageProcessingError::UnsupportedFormat)?;
    let decode_slot = format.needs_detail_jpeg().then(|| {
        DERIVATIVE_DECODE
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    });
    let oriented = match format {
        SupportedFormat::Heic => heic::decode(bytes)?,
        SupportedFormat::Tiff => {
            let orientation = tiff::inspect(bytes)?;
            apply_orientation(decode_raster(bytes, ImageFormat::Tiff)?, orientation)
        }
        SupportedFormat::Raster(raster) => {
            apply_orientation(decode_raster(bytes, raster)?, read_orientation(bytes))
        }
    };

    Ok((decode_slot, oriented))
}

fn check_dimensions(width: u32, height: u32) -> Result<(), ImageProcessingError> {
    if width == 0 || height == 0 {
        Err(ImageProcessingError::DecodeError)
    } else if u64::from(width) * u64::from(height) > MAX_MEGAPIXELS {
        Err(ImageProcessingError::TooLarge)
    } else {
        Ok(())
    }
}

fn decode_raster(bytes: &[u8], format: ImageFormat) -> Result<DynamicImage, ImageProcessingError> {
    if format != ImageFormat::Tiff && image::guess_format(bytes).ok() != Some(format) {
        return Err(ImageProcessingError::UnsupportedFormat);
    }
    let reader = || ImageReader::with_format(Cursor::new(bytes), format);
    let (width, height) = reader()
        .into_dimensions()
        .map_err(|_| ImageProcessingError::DecodeError)?;
    check_dimensions(width, height)?;
    reader().decode().map_err(|error| match error {
        image::ImageError::Limits(_) => ImageProcessingError::TooLarge,
        _ => ImageProcessingError::DecodeError,
    })
}

fn read_orientation(bytes: &[u8]) -> u32 {
    let Ok(exif) = Reader::new().read_from_container(&mut Cursor::new(bytes)) else {
        return 1;
    };

    exif.get_field(Tag::Orientation, In::PRIMARY)
        .and_then(|field| field.value.get_uint(0))
        .unwrap_or(1)
}

pub(crate) fn load_capture_details(path: &Path) -> io::Result<CaptureDetails> {
    let file = File::open(path)?;
    if file.metadata()?.len() > MAX_FILE_SIZE_BYTES {
        return Ok(CaptureDetails::default());
    }
    let mut original = BufReader::new(file);
    Ok(Reader::new()
        .read_from_container(&mut original)
        .ok()
        .as_ref()
        .map(read_capture_details)
        .unwrap_or_default())
}

fn read_capture_details(exif: &Exif) -> CaptureDetails {
    let text = |tag| match &exif.get_field(tag, In::PRIMARY)?.value {
        Value::Ascii(values) => {
            let value = String::from_utf8_lossy(values.first()?);
            let value = value
                .trim_matches(|character: char| character.is_whitespace() || character == '\0');
            (!value.is_empty()).then(|| value.to_owned())
        }
        _ => None,
    };
    let exposure = |tag| {
        let field = exif.get_field(tag, In::PRIMARY)?;
        match &field.value {
            Value::Rational(values)
                if values
                    .first()
                    .is_some_and(|value| value.num > 0 && value.denom > 0) =>
            {
                Some(field.display_value().with_unit(exif).to_string())
            }
            _ => None,
        }
    };
    let model = text(Tag::Model);
    let make = text(Tag::Make);
    let camera = match (make, model) {
        (Some(make), Some(model)) if !model.to_lowercase().starts_with(&make.to_lowercase()) => {
            Some(format!("{make} {model}"))
        }
        (make, model) => model.or(make),
    };
    let captured_at = text(Tag::DateTimeOriginal)
        .and_then(|value| chrono::NaiveDateTime::parse_from_str(&value, "%Y:%m:%d %H:%M:%S").ok())
        .map(|date| date.format("%Y-%m-%d %H:%M:%S").to_string());

    CaptureDetails {
        camera,
        lens: text(Tag::LensModel),
        captured_at,
        aperture: exposure(Tag::FNumber),
        shutter_speed: exposure(Tag::ExposureTime),
        iso: exif
            .get_field(Tag::PhotographicSensitivity, In::PRIMARY)
            .and_then(|field| field.value.get_uint(0))
            .filter(|iso| *iso > 0),
    }
}

pub fn apply_orientation(image: DynamicImage, orientation: u32) -> DynamicImage {
    match orientation {
        2 => image.fliph(),
        3 => image.rotate180(),
        4 => image.flipv(),
        5 => image.rotate90().fliph(),
        6 => image.rotate90(),
        7 => image.rotate270().fliph(),
        8 => image.rotate270(),
        _ => image,
    }
}

fn render_jpeg(
    image: &DynamicImage,
    max_dimension: u32,
    quality: u8,
) -> Result<Vec<u8>, ImageProcessingError> {
    let rgb = if image.width().max(image.height()) > max_dimension {
        image.thumbnail(max_dimension, max_dimension).into_rgb8()
    } else {
        image.to_rgb8()
    };
    let mut buffer = Cursor::new(Vec::new());
    let mut encoder = JpegEncoder::new_with_quality(&mut buffer, quality);
    encoder
        .encode(
            rgb.as_raw(),
            rgb.width(),
            rgb.height(),
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|_| ImageProcessingError::PreviewError)?;

    Ok(buffer.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{GrayImage, Luma};

    // A 2x3 (w x h) image where every pixel is distinct, so any transform is
    // uniquely identifiable:
    //   (0,0)=10 (1,0)=20
    //   (0,1)=30 (1,1)=40
    //   (0,2)=50 (1,2)=60
    fn sample() -> DynamicImage {
        let mut img = GrayImage::new(2, 3);
        let mut value = 10u8;
        for y in 0..3 {
            for x in 0..2 {
                img.put_pixel(x, y, Luma([value]));
                value += 10;
            }
        }
        DynamicImage::ImageLuma8(img)
    }

    fn as_rows(image: &DynamicImage) -> Vec<Vec<u8>> {
        let luma = image.to_luma8();
        (0..luma.height())
            .map(|y| (0..luma.width()).map(|x| luma.get_pixel(x, y)[0]).collect())
            .collect()
    }

    #[test]
    fn orientation_5_is_transpose() {
        let out = apply_orientation(sample(), 5);
        assert_eq!((out.width(), out.height()), (3, 2));
        assert_eq!(as_rows(&out), vec![vec![10, 30, 50], vec![20, 40, 60]]);
    }

    #[test]
    fn orientation_7_is_transverse() {
        let out = apply_orientation(sample(), 7);
        assert_eq!((out.width(), out.height()), (3, 2));
        assert_eq!(as_rows(&out), vec![vec![60, 40, 20], vec![50, 30, 10]]);
    }

    #[test]
    fn orientation_6_and_8_are_opposite_quarter_turns() {
        assert_eq!(
            as_rows(&apply_orientation(sample(), 6)),
            vec![vec![50, 30, 10], vec![60, 40, 20]]
        );
        assert_eq!(
            as_rows(&apply_orientation(sample(), 8)),
            vec![vec![20, 40, 60], vec![10, 30, 50]]
        );
    }

    #[test]
    fn orientation_1_to_4_keep_dimensions() {
        for orientation in 1..=4 {
            let out = apply_orientation(sample(), orientation);
            assert_eq!((out.width(), out.height()), (2, 3));
        }
    }

    #[test]
    fn supported_extension_is_case_insensitive() {
        for extension in ["JPG", "JpEg", "PnG", "WebP", "HEIC", "HeiF", "TIF", "TiFf"] {
            assert!(is_supported_extension(Path::new(&format!(
                "photo.{extension}"
            ))));
        }
        for extension in ["dng", "arw", "avif", "gif", "bmp", "jxl", "txt"] {
            assert!(!is_supported_extension(Path::new(&format!(
                "photo.{extension}"
            ))));
        }
    }

    #[test]
    fn jpeg_png_and_webp_still_prepare_without_detail_derivatives() {
        for (name, format) in [
            ("one.jpg", ImageFormat::Jpeg),
            ("two.png", ImageFormat::Png),
            ("three.webp", ImageFormat::WebP),
        ] {
            let mut bytes = Cursor::new(Vec::new());
            sample().into_rgb8().write_to(&mut bytes, format).unwrap();
            let prepared = prepare_photograph(Path::new(name), bytes.get_ref()).unwrap();
            assert_eq!((prepared.width, prepared.height), (2, 3));
            assert_eq!(jpeg_dimensions(&prepared.thumbnail_jpeg), (2, 3));
            assert!(!uses_detail_cache(Path::new(name)));
            assert_eq!(
                prepare_photograph(Path::new("renamed.heic"), bytes.get_ref()),
                Err(ImageProcessingError::UnsupportedFormat)
            );
            assert_eq!(
                prepare_photograph(Path::new("renamed.tiff"), bytes.get_ref()),
                Err(ImageProcessingError::UnsupportedFormat)
            );
        }
    }

    fn jpeg_dimensions(bytes: &[u8]) -> (u32, u32) {
        assert_eq!(image::guess_format(bytes).unwrap(), ImageFormat::Jpeg);
        let decoded = image::load_from_memory(bytes).unwrap();
        (decoded.width(), decoded.height())
    }

    #[test]
    fn heic_applies_item_orientation_once_and_accepts_compatible_heif() {
        let bytes = include_bytes!("../tests/fixtures/rotated.heic");
        let mut generic = bytes.to_vec();
        generic[8..12].copy_from_slice(b"mif1");
        for (name, bytes) in [
            ("rotated.HEIC", bytes.as_slice()),
            ("generic.HEIF", generic.as_slice()),
        ] {
            let photo = prepare_photograph(Path::new(name), bytes).unwrap();
            assert_eq!((photo.width, photo.height), (48, 64));
            assert_eq!(jpeg_dimensions(&photo.thumbnail_jpeg), (48, 64));
            assert_eq!(
                jpeg_dimensions(&prepare_detail_jpeg(Path::new(name), bytes).unwrap()),
                (48, 64)
            );
        }
        generic[8..12].copy_from_slice(b"avif");
        assert_eq!(
            prepare_photograph(Path::new("wrong.heif"), &generic),
            Err(ImageProcessingError::UnsupportedFormat)
        );
        assert_eq!(
            prepare_photograph(Path::new("truncated.heic"), &bytes[..bytes.len() - 1]),
            Err(ImageProcessingError::DecodeError)
        );
    }

    #[test]
    fn tiff_renders_first_page_with_orientation_and_bounded_derivatives() {
        use ::tiff::{
            encoder::{colortype, TiffEncoder},
            tags::Tag,
        };

        let mut data = Cursor::new(Vec::new());
        let mut encoder = TiffEncoder::new(&mut data).unwrap();
        let mut first = encoder.new_image::<colortype::RGB16>(2700, 1350).unwrap();
        first.encoder().write_tag(Tag::Orientation, 6_u16).unwrap();
        first.write_data(&vec![20000; 2700 * 1350 * 3]).unwrap();
        encoder.write_image::<colortype::Gray8>(1, 1, &[5]).unwrap();
        let photo = prepare_photograph(Path::new("photo.TIF"), data.get_ref()).unwrap();
        assert_eq!((photo.width, photo.height), (1350, 2700));
        assert_eq!(jpeg_dimensions(&photo.thumbnail_jpeg), (320, 640));
        assert_eq!(
            jpeg_dimensions(&prepare_detail_jpeg(Path::new("photo.TIF"), data.get_ref()).unwrap()),
            (1280, 2560)
        );
    }

    #[test]
    fn tiff_rejects_a_renderable_raw_preview_even_with_a_tiff_extension() {
        use ::tiff::{
            encoder::{colortype, TiffEncoder},
            tags::Tag,
        };

        for tag in [Tag::Unknown(50706), Tag::Unknown(33422)] {
            for raw_in_first_page in [true, false] {
                let mut bytes = Cursor::new(Vec::new());
                let mut encoder = TiffEncoder::new(&mut bytes).unwrap();
                if !raw_in_first_page {
                    encoder.write_image::<colortype::Gray8>(1, 1, &[5]).unwrap();
                }
                let mut image = encoder.new_image::<colortype::Gray8>(1, 1).unwrap();
                image
                    .encoder()
                    .write_tag(tag, &[1_u8, 4, 0, 0][..])
                    .unwrap();
                image.write_data(&[5]).unwrap();
                assert_eq!(
                    prepare_photograph(Path::new("renamed.tiff"), bytes.get_ref()),
                    Err(ImageProcessingError::UnsupportedFormat)
                );
            }
        }
    }

    #[test]
    fn tiff_rejects_excessive_dimensions_before_pixel_decode() {
        use ::tiff::{
            encoder::{colortype, TiffEncoder},
            tags::Tag,
        };

        let mut bytes = Cursor::new(Vec::new());
        let mut encoder = TiffEncoder::new(&mut bytes).unwrap();
        let mut image = encoder.new_image::<colortype::Gray8>(2, 3).unwrap();
        image
            .encoder()
            .write_tag(Tag::ImageWidth, 60_000_000_u32)
            .unwrap();
        image.write_data(&[5; 6]).unwrap();
        assert_eq!(
            prepare_photograph(Path::new("large.tif"), bytes.get_ref()),
            Err(ImageProcessingError::TooLarge)
        );
    }

    #[test]
    fn bigtiff_support_does_not_bypass_subifd_raw_checks() {
        use ::tiff::{
            encoder::{colortype, Ifd8, TiffEncoder},
            tags::Tag,
        };

        for raw in [false, true] {
            let mut bytes = Cursor::new(Vec::new());
            let mut encoder = TiffEncoder::new_big(&mut bytes).unwrap();
            let mut subifd = encoder.extra_directory().unwrap();
            subifd
                .write_tag(
                    Tag::PhotometricInterpretation,
                    if raw { 32803_u16 } else { 1 },
                )
                .unwrap();
            let offsets = subifd.finish_with_offsets().unwrap();
            let mut first = encoder.new_image::<colortype::Gray8>(1, 1).unwrap();
            first
                .encoder()
                .write_tag(Tag::SubIfd, Ifd8(offsets.pointer.0))
                .unwrap();
            first.write_data(&[5]).unwrap();
            let prepared = prepare_photograph(Path::new("big.tiff"), bytes.get_ref());
            if raw {
                assert_eq!(prepared, Err(ImageProcessingError::UnsupportedFormat));
            } else {
                assert_eq!(jpeg_dimensions(&prepared.unwrap().thumbnail_jpeg), (1, 1));
            }
        }
    }
}
