use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::sync::atomic::AtomicBool;

use exif::{Exif, In, Reader, Tag, Value};
use serde::Serialize;

use super::{diagnostics::timed, import::scan_source, CatalogError};

const MAX_SAMPLES: usize = 24;
const MIN_ROLL_SAMPLES: usize = 5;
const MAX_HEADER_BYTES: u64 = 256 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SourceMediumHint {
    Noritsu,
    PossibleFilmRoll,
    NoneFound,
}

pub fn inspect_source_medium(source: &Path) -> Result<SourceMediumHint, CatalogError> {
    if cfg!(debug_assertions) {
        eprintln!("[photart-import] Source check started (before Import)");
    }
    timed("Source check total", || inspect_source(source))
}

fn inspect_source(source: &Path) -> Result<SourceMediumHint, CatalogError> {
    let scan = timed("Source check: folder scan", || {
        scan_source(source, &AtomicBool::new(false))
    })?;
    let candidates: Vec<_> = scan
        .candidates
        .iter()
        .map(Path::new)
        .filter(|path| {
            !path
                .file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with("._"))
        })
        .collect();
    Ok(timed("Source check: film detection", || {
        detect_medium(source, &candidates)
    }))
}

fn detect_medium(source: &Path, candidates: &[&Path]) -> SourceMediumHint {
    let sample_count = candidates.len().min(MAX_SAMPLES);
    let mut all_noritsu = sample_count > 0;
    let mut consistent_roll = sample_count >= MIN_ROLL_SAMPLES;
    let mut first_signature = None;

    for index in 0..sample_count {
        let relative = candidates[index * (candidates.len() - 1) / (sample_count - 1).max(1)];
        let path = source.join(relative);
        if all_noritsu {
            all_noritsu = is_noritsu(&path);
        }
        if consistent_roll {
            let signature = roll_filename(relative)
                .then(|| jpeg_signature(&path))
                .flatten();
            consistent_roll = match (first_signature.as_ref(), signature) {
                (Some(first), Some(signature)) => *first == signature,
                (None, Some(signature)) => {
                    first_signature = Some(signature);
                    true
                }
                (_, None) => false,
            };
        }
        if !all_noritsu && !consistent_roll {
            break;
        }
    }

    if all_noritsu {
        SourceMediumHint::Noritsu
    } else if consistent_roll {
        SourceMediumHint::PossibleFilmRoll
    } else {
        SourceMediumHint::NoneFound
    }
}

fn is_noritsu(path: &Path) -> bool {
    File::open(path)
        .ok()
        .and_then(|file| {
            Reader::new()
                .read_from_container(&mut BufReader::new(file.take(MAX_HEADER_BYTES)))
                .ok()
        })
        .is_some_and(|exif| {
            matches_exif(&exif, Tag::Make, "NORITSU KOKI")
                && matches_exif(&exif, Tag::Model, "EZ Controller")
        })
}

fn matches_exif(exif: &Exif, tag: Tag, expected: &str) -> bool {
    exif.get_field(tag, In::PRIMARY)
        .and_then(|field| match &field.value {
            Value::Ascii(values) => values.first(),
            _ => None,
        })
        .is_some_and(|value| {
            let value = String::from_utf8_lossy(value);
            value
                .trim_matches(|character: char| {
                    character.is_ascii_whitespace() || character == '\0'
                })
                .split_ascii_whitespace()
                .map(str::to_ascii_lowercase)
                .eq(expected
                    .split_ascii_whitespace()
                    .map(str::to_ascii_lowercase))
        })
}

fn roll_filename(path: &Path) -> bool {
    fn digits(value: &str, count: usize) -> bool {
        value.len() == count && value.bytes().all(|byte| byte.is_ascii_digit())
    }

    let name = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let mut parts = name.split('-');
    match (
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
    ) {
        (Some(roll), Some(row), Some(index), Some(frame), None) => {
            roll.strip_prefix(['A', 'a'])
                .is_some_and(|value| digits(value, 6))
                && row
                    .strip_prefix(['R', 'r'])
                    .is_some_and(|value| digits(value, 1))
                && digits(index, 2)
                && (1..=2).any(|count| digits(frame.trim_end_matches(['A', 'a']), count))
        }
        _ => false,
    }
}

#[derive(PartialEq, Eq)]
struct JpegSignature {
    frame_header: [u8; 5],
    quantization_tables: Vec<u8>,
}

fn jpeg_signature(path: &Path) -> Option<JpegSignature> {
    let mut input = BufReader::new(File::open(path).ok()?.take(MAX_HEADER_BYTES));
    let mut marker = [0; 2];
    input.read_exact(&mut marker).ok()?;
    if marker != [0xff, 0xd8] {
        return None;
    }

    let mut frame_header = None;
    let mut quantization_tables = Vec::new();
    loop {
        input.read_exact(&mut marker).ok()?;
        if marker[0] != 0xff {
            return None;
        }
        while marker[1] == 0xff {
            input.read_exact(&mut marker[1..]).ok()?;
        }
        match marker[1] {
            0xda => break,
            0x00 | 0x01 | 0xd0..=0xd9 => return None,
            _ => {}
        }
        let mut length = [0; 2];
        input.read_exact(&mut length).ok()?;
        let mut segment = vec![0; usize::from(u16::from_be_bytes(length)).checked_sub(2)?];
        input.read_exact(&mut segment).ok()?;
        match marker[1] {
            0xc0..=0xc2 => frame_header = Some(segment.get(..5)?.try_into().ok()?),
            0xdb => quantization_tables.extend(segment),
            _ => {}
        }
    }

    (!quantization_tables.is_empty()).then_some(JpegSignature {
        frame_header: frame_header?,
        quantization_tables,
    })
}
