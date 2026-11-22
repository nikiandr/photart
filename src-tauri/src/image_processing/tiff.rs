use std::collections::HashSet;
use std::io::Cursor;

use tiff::decoder::Decoder;
use tiff::tags::Tag;

use super::ImageProcessingError;

const MAX_DIRECTORIES: usize = 64;
const RAW_TAGS: [Tag; 4] = [
    Tag::Unknown(50706),
    Tag::Unknown(50707),
    Tag::Unknown(33421),
    Tag::Unknown(33422),
];

pub(super) fn inspect(bytes: &[u8]) -> Result<u32, ImageProcessingError> {
    if !matches!(
        bytes.get(..4),
        Some(b"II\x2a\0" | b"MM\0\x2a" | b"II\x2b\0" | b"MM\0\x2b")
    ) || bytes.get(8..12) == Some(b"CR\x02\0")
    {
        return Err(ImageProcessingError::UnsupportedFormat);
    }

    let mut decoder =
        Decoder::new(Cursor::new(bytes)).map_err(|_| ImageProcessingError::DecodeError)?;
    let orientation = decoder
        .find_tag_unsigned::<u32>(Tag::Orientation)
        .ok()
        .flatten()
        .unwrap_or(1);
    let mut pending: Vec<_> = decoder.ifd_pointer().into_iter().collect();
    let mut visited = HashSet::new();
    while let Some(pointer) = pending.pop() {
        if !visited.insert(pointer.0) || visited.len() > MAX_DIRECTORIES {
            return Err(ImageProcessingError::DecodeError);
        }
        let directory = decoder
            .read_directory(pointer)
            .map_err(|_| ImageProcessingError::DecodeError)?;
        if RAW_TAGS.iter().any(|tag| directory.contains(*tag)) {
            return Err(ImageProcessingError::UnsupportedFormat);
        }
        let mut tags = decoder.read_directory_tags(&directory);
        if matches!(
            tags.find_tag_unsigned::<u32>(Tag::PhotometricInterpretation)
                .map_err(|_| ImageProcessingError::DecodeError)?,
            Some(32803 | 34892)
        ) {
            return Err(ImageProcessingError::UnsupportedFormat);
        }
        pending.extend(directory.next());
        if let Some(entry) = directory.get(Tag::SubIfd) {
            if entry.count() > MAX_DIRECTORIES as u64 {
                return Err(ImageProcessingError::DecodeError);
            }
            let offsets = tags
                .get_tag(Tag::SubIfd)
                .and_then(|value| value.into_ifd_vec())
                .map_err(|_| ImageProcessingError::DecodeError)?;
            pending.extend(offsets);
        }
    }
    Ok(orientation)
}
