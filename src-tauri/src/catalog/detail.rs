use std::fs;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use crate::image_processing;

use super::error::CatalogError;
use super::names::validate_single_path_component_name;
use super::{manifest, paths, state};

static DETAIL_CACHE: Mutex<()> = Mutex::new(());

pub(super) fn lock_detail_cache() -> Result<MutexGuard<'static, ()>, CatalogError> {
    DETAIL_CACHE.lock().map_err(|_| cache_error())
}

pub(crate) fn get_photograph_detail(
    catalog: &Path,
    shoot_name: &str,
    relative_path: &str,
) -> Result<PathBuf, CatalogError> {
    let _cache = lock_detail_cache()?;
    let manifest = manifest::read_manifest(catalog)?;
    let shoot = state::find_shoot(&manifest, shoot_name)?;
    let photo = shoot
        .photographs
        .iter()
        .find(|photo| photo.relative_path == relative_path)
        .ok_or_else(|| {
            CatalogError::new(
                "photograph_not_found",
                "This photograph is no longer in the Shoot.",
            )
        })?;
    if !image_processing::uses_detail_cache(Path::new(relative_path)) {
        return Err(cache_error());
    }
    let shoot_path = catalog.join(paths::validate_relative_path(&shoot.relative_path)?);
    let source = state::original_path(catalog, &shoot_path, photo).ok_or_else(|| {
        CatalogError::new(
            "photograph_unavailable",
            "The original photograph is unavailable.",
        )
    })?;
    let catalog = paths::canonicalize_existing(catalog).map_err(|_| cache_error())?;
    let shoot_name = validate_single_path_component_name(shoot_name)?;
    let expected_parent = paths::shoot_thumbnails_dir(&catalog, &shoot_name);
    let parent = paths::authorize_catalog_path(&catalog, &expected_parent)?;
    if parent != expected_parent {
        return Err(cache_error());
    }
    let directory = parent.join(paths::DETAIL_DIR);
    match fs::create_dir(&directory) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
        Err(_) => return Err(cache_error()),
    }
    if !fs::symlink_metadata(&directory).is_ok_and(|metadata| metadata.is_dir()) {
        return Err(cache_error());
    }
    let path = directory.join(paths::thumbnail_file_name(relative_path));
    let cached = match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.is_file() => Some(metadata),
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        _ => return Err(cache_error()),
    };
    let modified = fs::metadata(&source)
        .and_then(|metadata| metadata.modified())
        .map_err(|_| cache_error())?;
    if !cached.is_some_and(|metadata| {
        metadata
            .modified()
            .is_ok_and(|cached_at| cached_at >= modified)
            && has_jpeg_markers(&path).unwrap_or(false)
    }) {
        let bytes = image_processing::read_source(&source).map_err(|_| cache_error())?;
        let jpeg = image_processing::prepare_detail_jpeg(Path::new(relative_path), &bytes)
            .map_err(|_| cache_error())?;
        let mut temporary =
            tempfile::NamedTempFile::new_in(&directory).map_err(|_| cache_error())?;
        temporary.write_all(&jpeg).map_err(|_| cache_error())?;
        temporary.persist(&path).map_err(|_| cache_error())?;
    }
    paths::authorize_catalog_path(&catalog, &path)
}

fn has_jpeg_markers(path: &Path) -> io::Result<bool> {
    let mut file = fs::File::open(path)?;
    let mut start = [0; 2];
    let mut end = [0; 2];
    file.read_exact(&mut start)?;
    file.seek(SeekFrom::End(-2))?;
    file.read_exact(&mut end)?;
    Ok(start == [0xff, 0xd8] && end == [0xff, 0xd9])
}

fn cache_error() -> CatalogError {
    CatalogError::new(
        "photograph_detail",
        "Could not prepare this photograph’s detail image.",
    )
}
