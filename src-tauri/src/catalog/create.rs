use std::fs;
use std::path::Path;

use crate::app_config::{AppConfigStore, CatalogPointer};

use super::error::CatalogError;
use super::manifest::{self, CatalogManifest};
use super::names::validate_single_path_component_name;
use super::paths;
use super::{CatalogSnapshot, CreateCatalogInput, ShootSummary};

pub fn create_catalog(
    input: CreateCatalogInput,
    config_store: &AppConfigStore,
    authorize: impl FnOnce(&Path) -> Result<(), CatalogError>,
) -> Result<CatalogSnapshot, CatalogError> {
    let name = validate_single_path_component_name(&input.name)?;

    if !input.parent_directory.exists() {
        return Err(CatalogError::new(
            "parent_missing",
            format!(
                "Parent directory does not exist: {}",
                input.parent_directory.display()
            ),
        ));
    }

    if !input.parent_directory.is_dir() {
        return Err(CatalogError::new(
            "parent_not_directory",
            format!(
                "Parent path is not a directory: {}",
                input.parent_directory.display()
            ),
        ));
    }

    let catalog_path = input.parent_directory.join(&name);
    if catalog_path.exists() {
        return Err(CatalogError::new(
            "catalog_exists",
            format!("A Catalog already exists at {}.", catalog_path.display()),
        ));
    }

    create_catalog_directories(&catalog_path)?;

    let manifest = CatalogManifest::new(name.clone());
    if let Err(error) = manifest::write_manifest(&catalog_path, &manifest) {
        rollback_created_catalog(&catalog_path);
        return Err(error);
    }

    let absolute_path = paths::canonicalize_existing(&catalog_path).map_err(|error| {
        rollback_created_catalog(&catalog_path);
        CatalogError::new(
            "catalog_path",
            format!(
                "Could not resolve Catalog path at {}: {error}",
                catalog_path.display()
            ),
        )
    })?;

    let pointer = CatalogPointer {
        name: name.clone(),
        path: absolute_path.clone(),
    };

    if let Err(error) = config_store.set_catalog(Some(pointer)) {
        rollback_created_catalog(&catalog_path);
        return Err(CatalogError::new(
            "config_write",
            format!("Could not persist Catalog pointer: {}", error.message),
        ));
    }

    if let Err(error) = authorize(&absolute_path) {
        // Config already points at this Catalog; remove the folder and clear config.
        rollback_created_catalog(&catalog_path);
        let _ = config_store.set_catalog(None);
        return Err(error);
    }

    Ok(CatalogSnapshot {
        name,
        path: absolute_path,
        shoots: Vec::<ShootSummary>::new(),
    })
}

fn create_catalog_directories(catalog_path: &Path) -> Result<(), CatalogError> {
    fs::create_dir(catalog_path).map_err(|error| {
        CatalogError::new(
            "catalog_create",
            format!(
                "Could not create Catalog at {}: {error}",
                catalog_path.display()
            ),
        )
    })?;

    let photart = paths::photart_dir(catalog_path);
    fs::create_dir(&photart).map_err(|error| {
        rollback_created_catalog(catalog_path);
        CatalogError::new(
            "catalog_create",
            format!(
                "Could not create Catalog metadata directory at {}: {error}",
                photart.display()
            ),
        )
    })?;

    for subdirectory in [paths::THUMBNAILS_DIR, paths::IMPORTS_DIR] {
        let path = photart.join(subdirectory);
        fs::create_dir(&path).map_err(|error| {
            rollback_created_catalog(catalog_path);
            CatalogError::new(
                "catalog_create",
                format!(
                    "Could not create Catalog directory at {}: {error}",
                    path.display()
                ),
            )
        })?;
    }

    Ok(())
}

fn rollback_created_catalog(catalog_path: &Path) {
    let _ = fs::remove_dir_all(catalog_path);
}
