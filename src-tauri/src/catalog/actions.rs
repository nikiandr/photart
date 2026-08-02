use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use super::error::CatalogError;
use super::manifest::{self, ShootRecord};
use super::names::validate_single_path_component_name;
use super::paths;
use super::state::shoot_details_from_record;
use super::ShootDetails;

static ACTION_SEQUENCE: AtomicU64 = AtomicU64::new(1);

pub fn rename_shoot(
    catalog_path: &Path,
    shoot_name: &str,
    new_name: &str,
) -> Result<ShootDetails, CatalogError> {
    let new_name = validate_single_path_component_name(new_name)?;
    let mut manifest = manifest::read_manifest(catalog_path)?;
    let shoot_index = find_shoot_index(&manifest.shoots, shoot_name)?;

    if manifest.shoots[shoot_index].name == new_name {
        return shoot_details_from_record(catalog_path, &manifest.shoots[shoot_index]);
    }

    if manifest.shoots.iter().any(|shoot| shoot.name == new_name) {
        return Err(CatalogError::new(
            "shoot_exists",
            format!("A Shoot named \"{new_name}\" already exists in the Catalog."),
        ));
    }

    let previous = manifest.shoots[shoot_index].clone();
    let renamed = renamed_record(&previous, &new_name)?;
    let old_originals = catalog_path.join(&previous.relative_path);
    let new_originals = catalog_path.join(&new_name);
    let old_thumbnails = paths::shoot_thumbnails_dir(catalog_path, &previous.name);
    let new_thumbnails = paths::shoot_thumbnails_dir(catalog_path, &new_name);

    require_directory(&old_originals, "Shoot folder")?;
    ensure_destination_available(&old_originals, &new_originals, "Shoot folder")?;
    if old_thumbnails.exists() {
        ensure_destination_available(&old_thumbnails, &new_thumbnails, "thumbnail folder")?;
    }

    let staging_root = action_staging_path(catalog_path, "rename");
    let staged_originals = staging_root.join("originals");
    let staged_thumbnails = staging_root.join("thumbnails");
    fs::create_dir_all(&staging_root).map_err(|error| {
        CatalogError::new(
            "shoot_rename",
            format!(
                "Could not prepare to rename Shoot \"{}\": {error}",
                previous.name
            ),
        )
    })?;

    move_path(&old_originals, &staged_originals, "shoot_rename")?;
    let had_thumbnails = old_thumbnails.exists();
    if had_thumbnails {
        if let Err(error) = move_path(&old_thumbnails, &staged_thumbnails, "shoot_rename") {
            let _ = fs::rename(&staged_originals, &old_originals);
            let _ = fs::remove_dir_all(&staging_root);
            return Err(error);
        }
    }

    if let Err(error) = move_path(&staged_originals, &new_originals, "shoot_rename") {
        let _ = fs::rename(&staged_originals, &old_originals);
        if had_thumbnails {
            let _ = fs::rename(&staged_thumbnails, &old_thumbnails);
        }
        let _ = fs::remove_dir_all(&staging_root);
        return Err(error);
    }

    if had_thumbnails {
        if let Err(error) = move_path(&staged_thumbnails, &new_thumbnails, "shoot_rename") {
            let _ = fs::rename(&new_originals, &old_originals);
            let _ = fs::rename(&staged_thumbnails, &old_thumbnails);
            let _ = fs::remove_dir_all(&staging_root);
            return Err(error);
        }
    }

    manifest.shoots[shoot_index] = renamed.clone();
    if let Err(error) = manifest::write_manifest(catalog_path, &manifest) {
        let rollback_error = rollback_rename(
            &new_originals,
            &old_originals,
            had_thumbnails.then_some((&new_thumbnails, &old_thumbnails)),
        );
        let _ = fs::remove_dir_all(&staging_root);
        if let Some(rollback_error) = rollback_error {
            return Err(CatalogError::new(
                "shoot_rename_rollback",
                format!(
                    "{}; additionally, rollback failed: {rollback_error}",
                    error.message
                ),
            ));
        }
        return Err(error);
    }

    let _ = fs::remove_dir_all(&staging_root);
    shoot_details_from_record(catalog_path, &renamed)
}

pub fn purge_shoot(catalog_path: &Path, shoot_name: &str) -> Result<(), CatalogError> {
    let mut manifest = manifest::read_manifest(catalog_path)?;
    let shoot_index = find_shoot_index(&manifest.shoots, shoot_name)?;
    let shoot = manifest.shoots[shoot_index].clone();
    let originals = catalog_path.join(&shoot.relative_path);
    let thumbnails = paths::shoot_thumbnails_dir(catalog_path, &shoot.name);

    require_directory(&originals, "Shoot folder")?;

    let staging_root = action_staging_path(catalog_path, "purge");
    let staged_originals = staging_root.join("originals");
    let staged_thumbnails = staging_root.join("thumbnails");
    fs::create_dir_all(&staging_root).map_err(|error| {
        CatalogError::new(
            "shoot_purge",
            format!(
                "Could not prepare to purge Shoot \"{}\": {error}",
                shoot.name
            ),
        )
    })?;

    move_path(&originals, &staged_originals, "shoot_purge")?;
    let had_thumbnails = thumbnails.exists();
    if had_thumbnails {
        if let Err(error) = move_path(&thumbnails, &staged_thumbnails, "shoot_purge") {
            let _ = fs::rename(&staged_originals, &originals);
            let _ = fs::remove_dir_all(&staging_root);
            return Err(error);
        }
    }

    manifest.shoots.remove(shoot_index);
    if let Err(error) = manifest::write_manifest(catalog_path, &manifest) {
        let originals_rollback = fs::rename(&staged_originals, &originals).err();
        let thumbnails_rollback = if had_thumbnails {
            fs::rename(&staged_thumbnails, &thumbnails).err()
        } else {
            None
        };
        let _ = fs::remove_dir_all(&staging_root);

        if originals_rollback.is_some() || thumbnails_rollback.is_some() {
            return Err(CatalogError::new(
                "shoot_purge_rollback",
                format!(
                    "{}; additionally, the Shoot could not be fully restored.",
                    error.message
                ),
            ));
        }
        return Err(error);
    }

    // Once the manifest commit succeeds, the Shoot is no longer visible. Any cleanup
    // residue remains contained inside Photart's private import area.
    let _ = fs::remove_dir_all(&staging_root);
    Ok(())
}

fn find_shoot_index(shoots: &[ShootRecord], name: &str) -> Result<usize, CatalogError> {
    shoots
        .iter()
        .position(|shoot| shoot.name == name)
        .ok_or_else(|| {
            CatalogError::new(
                "shoot_not_found",
                format!("Shoot \"{name}\" was not found in the Catalog."),
            )
        })
}

fn renamed_record(shoot: &ShootRecord, new_name: &str) -> Result<ShootRecord, CatalogError> {
    let old_prefix = format!(
        "{}/{}/{}/",
        paths::PHOTART_DIR,
        paths::THUMBNAILS_DIR,
        shoot.name
    );
    let new_prefix = format!(
        "{}/{}/{new_name}/",
        paths::PHOTART_DIR,
        paths::THUMBNAILS_DIR
    );
    let mut renamed = shoot.clone();
    renamed.name = new_name.to_string();
    renamed.relative_path = new_name.to_string();

    for photograph in &mut renamed.photographs {
        let suffix = photograph
            .thumbnail_relative_path
            .strip_prefix(&old_prefix)
            .ok_or_else(|| {
                CatalogError::new(
                    "shoot_rename",
                    format!(
                        "Thumbnail path does not belong to Shoot \"{}\": {}",
                        shoot.name, photograph.thumbnail_relative_path
                    ),
                )
            })?;
        photograph.thumbnail_relative_path = format!("{new_prefix}{suffix}");
    }

    Ok(renamed)
}

fn action_staging_path(catalog_path: &Path, action: &str) -> PathBuf {
    let sequence = ACTION_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    paths::imports_dir(catalog_path).join(format!("{action}-{}-{sequence}", std::process::id()))
}

fn require_directory(path: &Path, label: &str) -> Result<(), CatalogError> {
    if !path.is_dir() {
        return Err(CatalogError::new(
            "shoot_files_missing",
            format!("{label} is missing at {}.", path.display()),
        ));
    }
    Ok(())
}

fn ensure_destination_available(
    source: &Path,
    destination: &Path,
    label: &str,
) -> Result<(), CatalogError> {
    if destination.exists() {
        let same_path = paths::canonicalize_existing(source).ok()
            == paths::canonicalize_existing(destination).ok();
        if !same_path {
            return Err(CatalogError::new(
                "shoot_folder_exists",
                format!("A {label} already exists at {}.", destination.display()),
            ));
        }
    }
    Ok(())
}

fn move_path(source: &Path, destination: &Path, code: &str) -> Result<(), CatalogError> {
    fs::rename(source, destination).map_err(|error| {
        CatalogError::new(
            code,
            format!(
                "Could not move {} to {}: {error}",
                source.display(),
                destination.display()
            ),
        )
    })
}

fn rollback_rename(
    new_originals: &Path,
    old_originals: &Path,
    thumbnails: Option<(&Path, &Path)>,
) -> Option<String> {
    let mut errors = Vec::new();
    if let Err(error) = fs::rename(new_originals, old_originals) {
        errors.push(error.to_string());
    }
    if let Some((new_thumbnails, old_thumbnails)) = thumbnails {
        if let Err(error) = fs::rename(new_thumbnails, old_thumbnails) {
            errors.push(error.to_string());
        }
    }
    (!errors.is_empty()).then(|| errors.join("; "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::manifest::{CatalogManifest, PhotographRecord};
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEST: AtomicU64 = AtomicU64::new(1);

    struct TestCatalog(PathBuf);

    impl TestCatalog {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "photart-actions-{}-{}",
                std::process::id(),
                NEXT_TEST.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(paths::imports_dir(&path)).unwrap();
            fs::create_dir_all(paths::thumbnails_dir(&path)).unwrap();
            Self(path)
        }

        fn add_shoot(&self, name: &str) {
            fs::create_dir_all(self.0.join(name)).unwrap();
            fs::write(self.0.join(name).join("frame.jpg"), b"original").unwrap();
            let thumbs = paths::shoot_thumbnails_dir(&self.0, name);
            fs::create_dir_all(&thumbs).unwrap();
            fs::write(thumbs.join("frame.jpg"), b"thumbnail").unwrap();

            let mut catalog = CatalogManifest::new("Test".into());
            catalog.shoots.push(ShootRecord {
                name: name.into(),
                relative_path: name.into(),
                imported_at: "2026-08-02T12:00:00Z".into(),
                photograph_count: 1,
                skipped_file_count: 0,
                photographs: vec![PhotographRecord {
                    relative_path: "frame.jpg".into(),
                    file_name: "frame.jpg".into(),
                    width: 10,
                    height: 10,
                    thumbnail_relative_path: format!(".photart/thumbnails/{name}/frame.jpg"),
                }],
            });
            manifest::write_manifest(&self.0, &catalog).unwrap();
        }
    }

    impl Drop for TestCatalog {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn rename_moves_files_and_updates_manifest_paths() {
        let catalog = TestCatalog::new();
        catalog.add_shoot("Before");

        let renamed = rename_shoot(&catalog.0, "Before", "After").unwrap();

        assert_eq!(renamed.summary.name, "After");
        assert!(catalog.0.join("After/frame.jpg").exists());
        assert!(!catalog.0.join("Before").exists());
        assert!(paths::shoot_thumbnails_dir(&catalog.0, "After")
            .join("frame.jpg")
            .exists());
        assert_eq!(
            renamed.photographs[0].thumbnail_relative_path,
            ".photart/thumbnails/After/frame.jpg"
        );
    }

    #[test]
    fn purge_removes_files_and_manifest_record() {
        let catalog = TestCatalog::new();
        catalog.add_shoot("To Purge");

        purge_shoot(&catalog.0, "To Purge").unwrap();

        assert!(!catalog.0.join("To Purge").exists());
        assert!(!paths::shoot_thumbnails_dir(&catalog.0, "To Purge").exists());
        assert!(manifest::read_manifest(&catalog.0)
            .unwrap()
            .shoots
            .is_empty());
    }

    #[test]
    fn rename_rejects_an_existing_shoot_name_without_moving_files() {
        let catalog = TestCatalog::new();
        catalog.add_shoot("Before");
        let mut manifest = manifest::read_manifest(&catalog.0).unwrap();
        let mut other = manifest.shoots[0].clone();
        other.name = "Existing".into();
        other.relative_path = "Existing".into();
        other.photographs[0].thumbnail_relative_path =
            ".photart/thumbnails/Existing/frame.jpg".into();
        fs::create_dir_all(catalog.0.join("Existing")).unwrap();
        fs::create_dir_all(paths::shoot_thumbnails_dir(&catalog.0, "Existing")).unwrap();
        manifest.shoots.push(other);
        manifest::write_manifest(&catalog.0, &manifest).unwrap();

        let error = rename_shoot(&catalog.0, "Before", "Existing").unwrap_err();

        assert_eq!(error.code, "shoot_exists");
        assert!(catalog.0.join("Before/frame.jpg").exists());
    }
}
