use std::path::{Path, PathBuf};

use crate::app_config::AppConfig;
use crate::image_processing::{self, CaptureDetails};

use super::error::CatalogError;
use super::manifest::{self, PhotographRecord, ShootRecord};
use super::paths;
use super::{CatalogSnapshot, CatalogState, PhotographEntry, ShootDetails, ShootSummary};

pub fn get_catalog_state(
    config: &AppConfig,
    pictures_directory: Option<PathBuf>,
) -> Result<CatalogState, CatalogError> {
    let Some(pointer) = &config.catalog else {
        return Ok(CatalogState::SetupRequired { pictures_directory });
    };

    let catalog_path = &pointer.path;

    if !catalog_path.exists() {
        return Ok(CatalogState::Unavailable {
            name: pointer.name.clone(),
            path: catalog_path.clone(),
            reason: format!(
                "The Catalog folder is missing at {}.",
                catalog_path.display()
            ),
            pictures_directory,
        });
    }

    if !catalog_path.is_dir() {
        return Ok(CatalogState::Unavailable {
            name: pointer.name.clone(),
            path: catalog_path.clone(),
            reason: format!(
                "The Catalog path is not a directory: {}.",
                catalog_path.display()
            ),
            pictures_directory,
        });
    }

    let manifest = match manifest::read_manifest(catalog_path) {
        Ok(manifest) => manifest,
        Err(error) => {
            return Ok(CatalogState::Unavailable {
                name: pointer.name.clone(),
                path: catalog_path.clone(),
                reason: error.message,
                pictures_directory,
            });
        }
    };

    if manifest.name != pointer.name {
        return Ok(CatalogState::Unavailable {
            name: pointer.name.clone(),
            path: catalog_path.clone(),
            reason: format!(
                "Catalog manifest name \"{}\" does not match configured name \"{}\".",
                manifest.name, pointer.name
            ),
            pictures_directory,
        });
    }

    let absolute_path =
        paths::canonicalize_existing(catalog_path).unwrap_or_else(|_| catalog_path.clone());

    Ok(CatalogState::Ready {
        catalog: CatalogSnapshot {
            name: manifest.name.clone(),
            path: absolute_path,
            shoots: manifest
                .sorted_shoots_newest_first()
                .into_iter()
                .map(shoot_summary_from_record)
                .collect(),
        },
    })
}

pub fn get_shoot(catalog_path: &Path, shoot_name: &str) -> Result<ShootDetails, CatalogError> {
    let manifest = manifest::read_manifest(catalog_path)?;
    shoot_details_from_record(catalog_path, find_shoot(&manifest, shoot_name)?)
}

fn find_shoot<'a>(
    manifest: &'a manifest::CatalogManifest,
    shoot_name: &str,
) -> Result<&'a ShootRecord, CatalogError> {
    manifest
        .shoots
        .iter()
        .find(|shoot| shoot.name == shoot_name)
        .ok_or_else(|| {
            CatalogError::new(
                "shoot_not_found",
                format!("Shoot \"{shoot_name}\" was not found in the Catalog."),
            )
        })
}

pub(crate) fn get_photograph_capture(
    catalog_path: &Path,
    catalog_name: &str,
    shoot_name: &str,
    relative_path: &str,
) -> Result<CaptureDetails, CatalogError> {
    let manifest = manifest::read_manifest(catalog_path)?;
    if manifest.name != catalog_name {
        return Err(CatalogError::new(
            "catalog_unavailable",
            "The Catalog name has changed.",
        ));
    }

    let shoot = find_shoot(&manifest, shoot_name)?;
    let photograph = shoot
        .photographs
        .iter()
        .find(|photograph| photograph.relative_path == relative_path)
        .ok_or_else(|| {
            CatalogError::new(
                "photograph_not_found",
                "This photograph is no longer in the Shoot.",
            )
        })?;
    let shoot_path = catalog_path.join(paths::validate_relative_path(&shoot.relative_path)?);
    let original = original_path(catalog_path, &shoot_path, photograph).ok_or_else(|| {
        CatalogError::new(
            "photograph_unavailable",
            "The original photograph is unavailable.",
        )
    })?;

    image_processing::load_capture_details(&original).map_err(|_| {
        CatalogError::new(
            "photograph_capture",
            "Could not read this photograph’s capture details.",
        )
    })
}

fn original_path(
    catalog_path: &Path,
    shoot_path: &Path,
    photo: &PhotographRecord,
) -> Option<PathBuf> {
    let path = shoot_path.join(paths::validate_relative_path(&photo.relative_path).ok()?);
    paths::authorize_catalog_path(catalog_path, &path)
        .ok()
        .filter(|path| path.is_file())
}

pub(crate) fn shoot_summary_from_record(shoot: &ShootRecord) -> ShootSummary {
    ShootSummary {
        name: shoot.name.clone(),
        relative_path: shoot.relative_path.clone(),
        imported_at: shoot.imported_at.clone(),
        photograph_count: shoot.photograph_count,
        skipped_file_count: shoot.skipped_file_count,
    }
}

pub(crate) fn shoot_details_from_record(
    catalog_path: &Path,
    shoot: &ShootRecord,
) -> Result<ShootDetails, CatalogError> {
    let shoot_path = catalog_path.join(paths::validate_relative_path(&shoot.relative_path)?);
    let mut photographs: Vec<PhotographEntry> = shoot
        .photographs
        .iter()
        .map(|photo| photograph_entry_from_record(catalog_path, &shoot_path, photo))
        .collect::<Result<Vec<_>, _>>()?;
    photographs.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));

    Ok(ShootDetails {
        summary: shoot_summary_from_record(shoot),
        photographs,
    })
}

fn photograph_entry_from_record(
    catalog_path: &Path,
    shoot_path: &Path,
    photo: &PhotographRecord,
) -> Result<PhotographEntry, CatalogError> {
    let absolute = manifest::absolute_thumbnail_path(catalog_path, &photo.thumbnail_relative_path);
    let thumbnail_absolute_path = paths::authorize_catalog_path(catalog_path, &absolute)?;

    Ok(PhotographEntry {
        relative_path: photo.relative_path.clone(),
        file_name: photo.file_name.clone(),
        width: photo.width,
        height: photo.height,
        thumbnail_relative_path: photo.thumbnail_relative_path.clone(),
        thumbnail_absolute_path,
        original_absolute_path: original_path(catalog_path, shoot_path, photo),
    })
}
