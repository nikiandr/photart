use std::fs::File;
use std::io::{self, BufReader, Cursor};
use std::path::Path;

use exif::{Exif, In, Reader, Tag, Value};
use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, ImageFormat, ImageReader};
use serde::Serialize;

pub const MAX_THUMBNAIL_DIMENSION: u32 = 640;
pub const THUMBNAIL_JPEG_QUALITY: u8 = 82;
const MAX_FILE_SIZE_BYTES: u64 = 100 * 1024 * 1024;
const MAX_MEGAPIXELS: u64 = 100_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedFormat {
    Jpeg,
    Png,
    WebP,
}

impl SupportedFormat {
    pub fn from_image_format(format: ImageFormat) -> Option<Self> {
        match format {
            ImageFormat::Jpeg => Some(Self::Jpeg),
            ImageFormat::Png => Some(Self::Png),
            ImageFormat::WebP => Some(Self::WebP),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedPhotograph {
    pub width: u32,
    pub height: u32,
    pub format: SupportedFormat,
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
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "jpg" | "jpeg" | "png" | "webp"
            )
        })
        .unwrap_or(false)
}

pub fn prepare_photograph(bytes: &[u8]) -> Result<PreparedPhotograph, ImageProcessingError> {
    if bytes.len() as u64 > MAX_FILE_SIZE_BYTES {
        return Err(ImageProcessingError::TooLarge);
    }

    let format = image::guess_format(bytes).map_err(|_| ImageProcessingError::UnsupportedFormat)?;
    let supported = SupportedFormat::from_image_format(format)
        .ok_or(ImageProcessingError::UnsupportedFormat)?;

    let (raw_width, raw_height) = {
        let mut reader = ImageReader::new(Cursor::new(bytes));
        reader.set_format(format);
        reader
            .into_dimensions()
            .map_err(|_| ImageProcessingError::DecodeError)?
    };

    let pixel_count = u64::from(raw_width) * u64::from(raw_height);
    if pixel_count > MAX_MEGAPIXELS {
        return Err(ImageProcessingError::TooLarge);
    }

    let image = image::load_from_memory_with_format(bytes, format)
        .map_err(|_| ImageProcessingError::DecodeError)?;
    let orientation = read_orientation(bytes);
    let (width, height) = oriented_dimensions(image.width(), image.height(), orientation);
    let oriented = apply_orientation(image, orientation);
    let thumbnail_jpeg = generate_thumbnail_jpeg(&oriented)?;

    Ok(PreparedPhotograph {
        width,
        height,
        format: supported,
        thumbnail_jpeg,
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
    let mut original = BufReader::new(File::open(path)?);
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

pub fn oriented_dimensions(width: u32, height: u32, orientation: u32) -> (u32, u32) {
    if (5..=8).contains(&orientation) {
        (height, width)
    } else {
        (width, height)
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

fn generate_thumbnail_jpeg(image: &DynamicImage) -> Result<Vec<u8>, ImageProcessingError> {
    let preview = if image.width().max(image.height()) > MAX_THUMBNAIL_DIMENSION {
        image.thumbnail(MAX_THUMBNAIL_DIMENSION, MAX_THUMBNAIL_DIMENSION)
    } else {
        image.clone()
    };

    let rgb = preview.to_rgb8();
    let mut buffer = Cursor::new(Vec::new());
    let mut encoder = JpegEncoder::new_with_quality(&mut buffer, THUMBNAIL_JPEG_QUALITY);
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
    fn oriented_dimensions_swaps_only_for_quarter_turns() {
        for orientation in [1, 2, 3, 4] {
            assert_eq!(oriented_dimensions(4032, 3024, orientation), (4032, 3024));
        }
        for orientation in [5, 6, 7, 8] {
            assert_eq!(oriented_dimensions(3024, 4032, orientation), (4032, 3024));
        }
    }

    #[test]
    fn supported_extension_is_case_insensitive() {
        assert!(is_supported_extension(Path::new("IMG_0001.JPG")));
        assert!(is_supported_extension(Path::new("shot.WebP")));
        assert!(!is_supported_extension(Path::new("notes.txt")));
    }

    #[test]
    fn prepare_photograph_reads_png_dimensions_and_thumbnail() {
        let mut bytes = Cursor::new(Vec::new());
        image::RgbImage::from_pixel(4, 6, image::Rgb([10, 20, 30]))
            .write_to(&mut bytes, ImageFormat::Png)
            .expect("fixture png should encode");

        let prepared = prepare_photograph(&bytes.into_inner()).expect("png should prepare");
        assert_eq!(prepared.width, 4);
        assert_eq!(prepared.height, 6);
        assert_eq!(prepared.format, SupportedFormat::Png);
        assert!(!prepared.thumbnail_jpeg.is_empty());
        assert_eq!(
            image::guess_format(&prepared.thumbnail_jpeg).ok(),
            Some(ImageFormat::Jpeg)
        );
    }
}
