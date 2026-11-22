use std::{ffi::c_void, marker::PhantomData, ptr::NonNull, slice};

use chrono::{DateTime, Local};
use image::{DynamicImage, ImageFormat, RgbImage};

use super::{
    apply_orientation, check_dimensions, decode_raster, read_orientation, CaptureDetails,
    ImageProcessingError, MAX_MEGAPIXELS,
};

#[repr(C)]
struct Metadata {
    make: [u8; 64],
    model: [u8; 64],
    lens: [u8; 128],
    timestamp: i64,
    iso: f32,
    aperture: f32,
    shutter: f32,
}

#[repr(C)]
struct PixelBuffer {
    data: *const u8,
    length: u32,
    width: u32,
    height: u32,
}

extern "C" {
    fn photart_raw_open(
        bytes: *const u8,
        length: usize,
        max_pixels: u64,
        status: *mut i32,
    ) -> *mut c_void;
    fn photart_raw_close(handle: *mut c_void);
    fn photart_raw_metadata(handle: *mut c_void, output: *mut Metadata);
    fn photart_raw_pixels(
        handle: *mut c_void,
        render_goal: u32,
        output: *mut PixelBuffer,
    ) -> *mut c_void;
    fn photart_raw_free_pixels(buffer: *mut c_void);
}

struct Decoder<'source>(NonNull<c_void>, PhantomData<&'source [u8]>);

impl<'source> Decoder<'source> {
    fn open(bytes: &'source [u8]) -> Result<Self, ImageProcessingError> {
        let mut status = 0;
        let handle =
            unsafe { photart_raw_open(bytes.as_ptr(), bytes.len(), MAX_MEGAPIXELS, &mut status) };
        NonNull::new(handle)
            .map(|handle| Self(handle, PhantomData))
            .ok_or(match status {
                1 => ImageProcessingError::UnsupportedFormat,
                2 => ImageProcessingError::TooLarge,
                _ => ImageProcessingError::DecodeError,
            })
    }

    fn pixels(&mut self, render_goal: u32) -> Option<Pixels> {
        let mut buffer = std::mem::MaybeUninit::uninit();
        let handle =
            unsafe { photart_raw_pixels(self.0.as_ptr(), render_goal, buffer.as_mut_ptr()) };
        NonNull::new(handle).map(|handle| Pixels(handle, unsafe { buffer.assume_init() }))
    }

    fn metadata(&self) -> Metadata {
        let mut metadata = std::mem::MaybeUninit::uninit();
        unsafe {
            photart_raw_metadata(self.0.as_ptr(), metadata.as_mut_ptr());
            metadata.assume_init()
        }
    }
}

impl Drop for Decoder<'_> {
    fn drop(&mut self) {
        unsafe { photart_raw_close(self.0.as_ptr()) }
    }
}

struct Pixels(NonNull<c_void>, PixelBuffer);

impl Pixels {
    fn bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.1.data, self.1.length as usize) }
    }

    fn into_rgb(self) -> Option<DynamicImage> {
        let PixelBuffer { width, height, .. } = self.1;
        check_dimensions(width, height).ok()?;
        RgbImage::from_raw(width, height, self.bytes().to_vec()).map(DynamicImage::ImageRgb8)
    }
}

impl Drop for Pixels {
    fn drop(&mut self) {
        unsafe { photart_raw_free_pixels(self.0.as_ptr()) }
    }
}

pub(super) fn decode(bytes: &[u8], goal: u32) -> Result<DynamicImage, ImageProcessingError> {
    let mut decoder = Decoder::open(bytes)?;
    let embedded = decoder.pixels(0).and_then(|pixels| {
        decode_raster(pixels.bytes(), ImageFormat::Jpeg)
            .ok()
            .map(|image| apply_orientation(image, read_orientation(pixels.bytes())))
    });
    let long_edge = |image: &DynamicImage| image.width().max(image.height());
    let rendered = if embedded
        .as_ref()
        .is_none_or(|image| long_edge(image) < goal)
    {
        decoder.pixels(goal).and_then(Pixels::into_rgb)
    } else {
        None
    };
    match (embedded, rendered) {
        (Some(embedded), Some(rendered)) if long_edge(&rendered) > long_edge(&embedded) => {
            Ok(rendered)
        }
        (Some(embedded), _) => Ok(embedded),
        (None, Some(rendered)) => Ok(rendered),
        (None, None) => Err(ImageProcessingError::PreviewError),
    }
}

pub(super) fn capture_details(bytes: &[u8]) -> Result<CaptureDetails, ImageProcessingError> {
    let metadata = Decoder::open(bytes)?.metadata();
    let text = |bytes: &[u8]| {
        let end = bytes
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(bytes.len());
        let text = String::from_utf8_lossy(&bytes[..end]);
        let text = text.trim();
        (!text.is_empty()).then(|| text.to_owned())
    };
    let camera = match (text(&metadata.make), text(&metadata.model)) {
        (Some(make), Some(model)) if !model.to_lowercase().starts_with(&make.to_lowercase()) => {
            Some(format!("{make} {model}"))
        }
        (make, model) => model.or(make),
    };
    let positive = |number: f32| number.is_finite() && number > 0.0;
    let shutter_speed = positive(metadata.shutter).then(|| {
        if metadata.shutter <= 0.5 {
            format!("1/{:.0} s", metadata.shutter.recip())
        } else {
            format!("{} s", metadata.shutter)
        }
    });
    Ok(CaptureDetails {
        camera,
        lens: text(&metadata.lens),
        captured_at: DateTime::from_timestamp(metadata.timestamp, 0)
            .filter(|_| metadata.timestamp > 0)
            .map(|time| {
                time.with_timezone(&Local)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string()
            }),
        aperture: positive(metadata.aperture).then(|| format!("f/{}", metadata.aperture)),
        shutter_speed,
        iso: positive(metadata.iso).then(|| metadata.iso.round() as u32),
    })
}
