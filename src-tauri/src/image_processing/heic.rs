use image::{DynamicImage, GrayAlphaImage, GrayImage, RgbImage, RgbaImage};
use zune_heic::zune_core::{bytestream::ZCursor, colorspace::ColorSpace, options::DecoderOptions};
use zune_heic::HeifDecoder;

use super::{check_dimensions, ImageProcessingError, MAX_FILE_SIZE_BYTES};

const MAX_AXIS: usize = 10_000;

pub(super) fn decode(bytes: &[u8]) -> Result<DynamicImage, ImageProcessingError> {
    validate_container(bytes)?;
    let options = DecoderOptions::default()
        .hvec_set_use_videotoolbox(false)
        .set_num_threads(1)
        .set_max_width(MAX_AXIS)
        .set_max_height(MAX_AXIS)
        .set_hevc_max_mdat_size(MAX_FILE_SIZE_BYTES as usize);
    let mut decoder = HeifDecoder::new_with_options(ZCursor::new(bytes), options);
    decoder
        .decode_headers()
        .map_err(|_| ImageProcessingError::DecodeError)?;
    let width = decoder.width().ok_or(ImageProcessingError::DecodeError)? as u32;
    let height = decoder.height().ok_or(ImageProcessingError::DecodeError)? as u32;
    check_dimensions(width, height)?;
    let pixels = decoder
        .decode()
        .map_err(|_| ImageProcessingError::DecodeError)?;

    match decoder.colorspace() {
        Some(ColorSpace::RGB) => {
            RgbImage::from_raw(width, height, pixels).map(DynamicImage::ImageRgb8)
        }
        Some(ColorSpace::RGBA) => {
            RgbaImage::from_raw(width, height, pixels).map(DynamicImage::ImageRgba8)
        }
        Some(ColorSpace::Luma) => {
            GrayImage::from_raw(width, height, pixels).map(DynamicImage::ImageLuma8)
        }
        Some(ColorSpace::LumaA) => {
            GrayAlphaImage::from_raw(width, height, pixels).map(DynamicImage::ImageLumaA8)
        }
        _ => None,
    }
    .ok_or(ImageProcessingError::DecodeError)
}

fn validate_container(mut bytes: &[u8]) -> Result<(), ImageProcessingError> {
    if bytes.get(4..8) != Some(b"ftyp") {
        return Err(ImageProcessingError::UnsupportedFormat);
    }

    let mut first = true;
    while !bytes.is_empty() {
        let (header, end) = box_bounds(bytes).ok_or(ImageProcessingError::DecodeError)?;
        if first {
            let brands = &bytes[header..end];
            if brands.len() < 8 || !brands.len().is_multiple_of(4) {
                return Err(ImageProcessingError::DecodeError);
            }
            let still_brands = brands
                .chunks_exact(4)
                .enumerate()
                .filter_map(|(index, brand)| (index != 1).then_some(brand));
            if !matches!(&brands[..4], b"heic" | b"heix" | b"mif1")
                || still_brands
                    .clone()
                    .any(|brand| matches!(brand, b"avif" | b"avis"))
            {
                return Err(ImageProcessingError::UnsupportedFormat);
            }
            first = false;
        }
        bytes = &bytes[end..];
    }
    Ok(())
}

fn box_bounds(bytes: &[u8]) -> Option<(usize, usize)> {
    let size = u32::from_be_bytes(bytes.first_chunk().copied()?);
    let (header, size) = match size {
        0 => (8, bytes.len()),
        1 => (
            16,
            usize::try_from(u64::from_be_bytes(bytes.get(8..16)?.try_into().ok()?)).ok()?,
        ),
        size => (8, size as usize),
    };
    (header <= size && size <= bytes.len()).then_some((header, size))
}
