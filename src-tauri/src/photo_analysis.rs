use std::fs;
use std::io::{Cursor, ErrorKind};
use std::path::Path;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use exif::{In, Reader, Tag, Value};
use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, ImageFormat, ImageReader};
use serde::Serialize;

const MAX_FILE_SIZE_BYTES: u64 = 100 * 1024 * 1024;
const MAX_MEGAPIXELS: u64 = 100_000_000;
const MAX_PREVIEW_DIMENSION: u32 = 1600;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotoAnalysis {
    pub file_name: String,
    pub file_size_bytes: u64,
    pub format: String,
    pub mime_type: String,
    pub width: u32,
    pub height: u32,
    pub color_type: String,
    pub preview_data_url: String,
    pub exif: PhotoExif,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotoExif {
    pub captured_at: Option<String>,
    pub camera_make: Option<String>,
    pub camera_model: Option<String>,
    pub lens_model: Option<String>,
    pub exposure_time: Option<String>,
    pub aperture: Option<String>,
    pub iso: Option<String>,
    pub focal_length: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotoAnalysisError {
    pub code: String,
    pub message: String,
}

impl PhotoAnalysisError {
    fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
        }
    }
}

struct ParsedExif {
    captured_at: Option<String>,
    camera_make: Option<String>,
    camera_model: Option<String>,
    lens_model: Option<String>,
    exposure_time: Option<String>,
    aperture: Option<String>,
    iso: Option<String>,
    focal_length: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    orientation: u32,
}

impl Default for ParsedExif {
    fn default() -> Self {
        Self {
            captured_at: None,
            camera_make: None,
            camera_model: None,
            lens_model: None,
            exposure_time: None,
            aperture: None,
            iso: None,
            focal_length: None,
            latitude: None,
            longitude: None,
            orientation: 1,
        }
    }
}

pub fn analyze(path: &str) -> Result<PhotoAnalysis, PhotoAnalysisError> {
    let path = Path::new(path);

    let metadata = fs::metadata(path).map_err(|error| {
        if error.kind() == ErrorKind::NotFound {
            PhotoAnalysisError::new("file_not_found", "The selected file could not be found.")
        } else {
            PhotoAnalysisError::new("file_access", "The selected file could not be read.")
        }
    })?;

    if !metadata.is_file() {
        return Err(PhotoAnalysisError::new(
            "file_access",
            "The selected path is not a file.",
        ));
    }

    let file_size_bytes = metadata.len();
    if file_size_bytes > MAX_FILE_SIZE_BYTES {
        return Err(PhotoAnalysisError::new(
            "file_too_large",
            "This photo exceeds the 100 MiB size limit.",
        ));
    }

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .to_string();

    let bytes = fs::read(path).map_err(|_| {
        PhotoAnalysisError::new("file_access", "The selected file could not be read.")
    })?;

    let format = image::guess_format(&bytes).map_err(|_| {
        PhotoAnalysisError::new(
            "unsupported_format",
            "Only JPEG, PNG, and WebP photos are supported.",
        )
    })?;

    let (format_label, mime_type) = match format {
        ImageFormat::Jpeg => ("jpeg", "image/jpeg"),
        ImageFormat::Png => ("png", "image/png"),
        ImageFormat::WebP => ("webp", "image/webp"),
        _ => {
            return Err(PhotoAnalysisError::new(
                "unsupported_format",
                "Only JPEG, PNG, and WebP photos are supported.",
            ));
        }
    };

    let (raw_width, raw_height) = {
        let mut reader = ImageReader::new(Cursor::new(&bytes));
        reader.set_format(format);
        reader.into_dimensions().map_err(|_| {
            PhotoAnalysisError::new(
                "decode_error",
                "The photo could not be decoded. It may be corrupt or incomplete.",
            )
        })?
    };

    let pixel_count = u64::from(raw_width) * u64::from(raw_height);
    if pixel_count > MAX_MEGAPIXELS {
        return Err(PhotoAnalysisError::new(
            "image_too_large",
            "This photo exceeds the 100 megapixel limit.",
        ));
    }

    let image = image::load_from_memory_with_format(&bytes, format).map_err(|_| {
        PhotoAnalysisError::new(
            "decode_error",
            "The photo could not be decoded. It may be corrupt or incomplete.",
        )
    })?;

    let parsed_exif = parse_exif(&bytes);
    let (width, height) =
        oriented_dimensions(image.width(), image.height(), parsed_exif.orientation);
    let oriented = apply_orientation(image, parsed_exif.orientation);
    let preview_data_url = generate_preview(&oriented)?;

    Ok(PhotoAnalysis {
        file_name,
        file_size_bytes,
        format: format_label.to_string(),
        mime_type: mime_type.to_string(),
        width,
        height,
        color_type: color_type_label(&oriented),
        preview_data_url,
        exif: PhotoExif {
            captured_at: parsed_exif.captured_at,
            camera_make: parsed_exif.camera_make,
            camera_model: parsed_exif.camera_model,
            lens_model: parsed_exif.lens_model,
            exposure_time: parsed_exif.exposure_time,
            aperture: parsed_exif.aperture,
            iso: parsed_exif.iso,
            focal_length: parsed_exif.focal_length,
            latitude: parsed_exif.latitude,
            longitude: parsed_exif.longitude,
        },
    })
}

fn parse_exif(bytes: &[u8]) -> ParsedExif {
    let mut parsed = ParsedExif::default();
    let Ok(exif) = Reader::new().read_from_container(&mut Cursor::new(bytes)) else {
        return parsed;
    };

    parsed.orientation = exif
        .get_field(Tag::Orientation, In::PRIMARY)
        .and_then(|field| field.value.get_uint(0))
        .unwrap_or(1);

    parsed.captured_at = exif
        .get_field(Tag::DateTimeOriginal, In::PRIMARY)
        .or_else(|| exif.get_field(Tag::DateTime, In::PRIMARY))
        .and_then(|field| ascii_string(&field.value));

    parsed.camera_make = exif
        .get_field(Tag::Make, In::PRIMARY)
        .and_then(|field| ascii_string(&field.value));

    parsed.camera_model = exif
        .get_field(Tag::Model, In::PRIMARY)
        .and_then(|field| ascii_string(&field.value));

    parsed.lens_model = exif
        .get_field(Tag::LensModel, In::PRIMARY)
        .and_then(|field| ascii_string(&field.value));

    parsed.exposure_time = exif
        .get_field(Tag::ExposureTime, In::PRIMARY)
        .map(format_exposure_time);

    parsed.aperture = exif
        .get_field(Tag::FNumber, In::PRIMARY)
        .map(format_aperture);

    parsed.iso = exif
        .get_field(Tag::PhotographicSensitivity, In::PRIMARY)
        .and_then(|field| field.value.get_uint(0))
        .map(|iso| iso.to_string());

    parsed.focal_length = exif
        .get_field(Tag::FocalLength, In::PRIMARY)
        .map(format_focal_length);

    let latitude_ref = exif
        .get_field(Tag::GPSLatitudeRef, In::PRIMARY)
        .and_then(|field| ascii_string(&field.value));
    let longitude_ref = exif
        .get_field(Tag::GPSLongitudeRef, In::PRIMARY)
        .and_then(|field| ascii_string(&field.value));

    parsed.latitude = exif
        .get_field(Tag::GPSLatitude, In::PRIMARY)
        .and_then(|field| parse_gps_coordinate(&field.value, latitude_ref.as_deref()));
    parsed.longitude = exif
        .get_field(Tag::GPSLongitude, In::PRIMARY)
        .and_then(|field| parse_gps_coordinate(&field.value, longitude_ref.as_deref()));

    parsed
}

fn ascii_string(value: &Value) -> Option<String> {
    let Value::Ascii(values) = value else {
        return None;
    };

    let text = values
        .iter()
        .map(|bytes| String::from_utf8_lossy(bytes))
        .collect::<Vec<_>>()
        .join("");
    let trimmed = text.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn parse_gps_coordinate(value: &Value, reference: Option<&str>) -> Option<f64> {
    let Value::Rational(rationals) = value else {
        return None;
    };

    if rationals.len() < 3 {
        return None;
    }

    let degrees = rationals[0].to_f64();
    let minutes = rationals[1].to_f64();
    let seconds = rationals[2].to_f64();
    let mut decimal = degrees + minutes / 60.0 + seconds / 3600.0;

    match reference {
        Some("S") | Some("W") => decimal = -decimal,
        Some(_) => {}
        None => {}
    }

    Some(decimal)
}

fn format_exposure_time(field: &exif::Field) -> String {
    match &field.value {
        Value::Rational(rationals) if !rationals.is_empty() => {
            let rational = rationals[0];
            if rational.denom == 0 {
                return field.display_value().to_string();
            }

            if rational.num >= rational.denom {
                format!("{:.3}s", rational.to_f64())
            } else {
                format!(
                    "1/{}s",
                    (rational.denom as f64 / rational.num as f64).round() as u64
                )
            }
        }
        _ => field.display_value().to_string(),
    }
}

fn format_aperture(field: &exif::Field) -> String {
    match &field.value {
        Value::Rational(rationals) if !rationals.is_empty() => {
            format!("f/{:.1}", rationals[0].to_f64())
        }
        _ => field.display_value().to_string(),
    }
}

fn format_focal_length(field: &exif::Field) -> String {
    match &field.value {
        Value::Rational(rationals) if !rationals.is_empty() => {
            format!("{:.0} mm", rationals[0].to_f64())
        }
        _ => field.display_value().to_string(),
    }
}

fn oriented_dimensions(width: u32, height: u32, orientation: u32) -> (u32, u32) {
    if (5..=8).contains(&orientation) {
        (height, width)
    } else {
        (width, height)
    }
}

fn apply_orientation(image: DynamicImage, orientation: u32) -> DynamicImage {
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

fn color_type_label(image: &DynamicImage) -> String {
    match image.color() {
        image::ColorType::L8 => "L8".to_string(),
        image::ColorType::La8 => "La8".to_string(),
        image::ColorType::Rgb8 => "Rgb8".to_string(),
        image::ColorType::Rgba8 => "Rgba8".to_string(),
        image::ColorType::L16 => "L16".to_string(),
        image::ColorType::La16 => "La16".to_string(),
        image::ColorType::Rgb16 => "Rgb16".to_string(),
        image::ColorType::Rgba16 => "Rgba16".to_string(),
        image::ColorType::Rgb32F => "Rgb32F".to_string(),
        image::ColorType::Rgba32F => "Rgba32F".to_string(),
        _ => "Unknown".to_string(),
    }
}

fn generate_preview(image: &DynamicImage) -> Result<String, PhotoAnalysisError> {
    let preview = if image.width().max(image.height()) > MAX_PREVIEW_DIMENSION {
        image.thumbnail(MAX_PREVIEW_DIMENSION, MAX_PREVIEW_DIMENSION)
    } else {
        image.clone()
    };

    let rgb = preview.to_rgb8();
    let mut buffer = Cursor::new(Vec::new());
    let mut encoder = JpegEncoder::new_with_quality(&mut buffer, 85);
    encoder
        .encode(
            rgb.as_raw(),
            rgb.width(),
            rgb.height(),
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|_| {
            PhotoAnalysisError::new(
                "preview_error",
                "A preview could not be generated for this photo.",
            )
        })?;

    let encoded = STANDARD.encode(buffer.into_inner());
    Ok(format!("data:image/jpeg;base64,{encoded}"))
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

    // Orientation 5 is a transpose (reflection across the main diagonal), and
    // orientation 7 is a transverse (reflection across the anti-diagonal). These
    // two were previously swapped, which flipped front-camera photos (stored
    // portrait, tagged orientation 5) upside down.
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
        // 6 = rotate 90 CW, 8 = rotate 90 CCW.
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
    fn ascii_string_reads_raw_bytes_without_quotes() {
        let value = Value::Ascii(vec![b"Canon".to_vec()]);
        assert_eq!(ascii_string(&value).as_deref(), Some("Canon"));
    }

    #[test]
    fn gps_southern_and_western_refs_negate() {
        use exif::Rational;

        let coordinate = Value::Rational(vec![
            Rational { num: 37, denom: 1 },
            Rational { num: 46, denom: 1 },
            Rational { num: 30, denom: 1 },
        ]);

        let north = parse_gps_coordinate(&coordinate, Some("N")).unwrap();
        let south = parse_gps_coordinate(&coordinate, Some("S")).unwrap();
        let east = parse_gps_coordinate(&coordinate, Some("E")).unwrap();
        let west = parse_gps_coordinate(&coordinate, Some("W")).unwrap();

        assert!(north > 0.0);
        assert_eq!(south, -north);
        assert!(east > 0.0);
        assert_eq!(west, -east);
    }
}
