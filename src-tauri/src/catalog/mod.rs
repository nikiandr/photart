//! Photart Catalog: creation, validation, manifest access, import, and path authorization.

mod actions;
mod create;
mod error;
mod import;
mod manifest;
mod names;
mod paths;
mod state;

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub use actions::{purge_shoot, rename_shoot};
#[allow(unused_imports)] // re-exported for Tauri commands
pub use create::create_catalog;
#[allow(unused_imports)]
pub use error::CatalogError;
#[allow(unused_imports)]
pub use import::{import_shoot, ImportCoordinator, ImportPermit};
#[allow(unused_imports)]
pub use names::validate_single_path_component_name;
#[allow(unused_imports)]
pub use paths::{authorize_catalog_path, is_path_inside_catalog};
#[allow(unused_imports)]
pub use state::{get_catalog_state, get_shoot};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum CatalogState {
    SetupRequired {
        pictures_directory: Option<PathBuf>,
    },
    Unavailable {
        name: String,
        path: PathBuf,
        reason: String,
        pictures_directory: Option<PathBuf>,
    },
    Ready {
        catalog: CatalogSnapshot,
    },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogSnapshot {
    pub name: String,
    pub path: PathBuf,
    pub shoots: Vec<ShootSummary>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShootSummary {
    pub name: String,
    pub relative_path: String,
    pub imported_at: String,
    pub photograph_count: u32,
    pub skipped_file_count: u32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShootDetails {
    pub summary: ShootSummary,
    pub photographs: Vec<PhotographEntry>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotographEntry {
    pub relative_path: String,
    pub file_name: String,
    pub width: u32,
    pub height: u32,
    pub thumbnail_relative_path: String,
    pub thumbnail_absolute_path: PathBuf,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCatalogInput {
    pub name: String,
    pub parent_directory: PathBuf,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportProgress {
    pub phase: String,
    pub total: Option<u32>,
    pub prepared: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportShootResult {
    pub shoot: ShootDetails,
    pub skipped_file_count: u32,
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    use image::{ImageFormat, Rgb, RgbImage};

    use crate::app_config::{AppConfig, AppConfigStore, CatalogPointer};

    use super::import::{import_shoot_inner, ImportFaults};
    use super::manifest::{self, CatalogManifest};
    use super::paths;
    use super::*;

    static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new(label: &str) -> Self {
            let sequence = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "photart-catalog-{label}-{}-{sequence}",
                std::process::id()
            ));
            std::fs::create_dir_all(&path).expect("test directory should be created");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn open_config(directory: &TestDirectory) -> AppConfigStore {
        AppConfigStore::open(directory.path().join("config.json")).expect("config should open")
    }

    fn write_png(path: &Path, width: u32, height: u32) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let mut bytes = Cursor::new(Vec::new());
        RgbImage::from_pixel(width, height, Rgb([10, 20, 30]))
            .write_to(&mut bytes, ImageFormat::Png)
            .expect("png should encode");
        std::fs::write(path, bytes.into_inner()).unwrap();
    }

    fn write_jpeg(path: &Path, width: u32, height: u32) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let mut bytes = Cursor::new(Vec::new());
        RgbImage::from_pixel(width, height, Rgb([40, 50, 60]))
            .write_to(&mut bytes, ImageFormat::Jpeg)
            .expect("jpeg should encode");
        std::fs::write(path, bytes.into_inner()).unwrap();
    }

    fn create_ready_catalog(name: &str) -> (TestDirectory, TestDirectory, PathBuf, AppConfigStore) {
        let parent = TestDirectory::new("parent");
        let config_dir = TestDirectory::new("config");
        let store = open_config(&config_dir);
        let snapshot = create_catalog(
            CreateCatalogInput {
                name: name.into(),
                parent_directory: parent.path().to_path_buf(),
            },
            &store,
            |_| Ok(()),
        )
        .expect("catalog should create");
        (parent, config_dir, snapshot.path, store)
    }

    #[test]
    fn creates_catalog_with_expected_layout() {
        let parent = TestDirectory::new("create");
        let config_dir = TestDirectory::new("create-config");
        let store = open_config(&config_dir);

        let snapshot = create_catalog(
            CreateCatalogInput {
                name: "Photart Catalog".into(),
                parent_directory: parent.path().to_path_buf(),
            },
            &store,
            |_| Ok(()),
        )
        .expect("create should succeed");

        assert_eq!(snapshot.name, "Photart Catalog");
        assert!(snapshot.path.exists());
        assert!(paths::manifest_path(&snapshot.path).exists());
        assert!(paths::thumbnails_dir(&snapshot.path).is_dir());
        assert!(paths::imports_dir(&snapshot.path).is_dir());

        let config = store.get().unwrap();
        assert_eq!(
            config.catalog,
            Some(CatalogPointer {
                name: "Photart Catalog".into(),
                path: snapshot.path.clone(),
            })
        );
    }

    #[test]
    fn rejects_invalid_names_and_destination_collision() {
        let parent = TestDirectory::new("names");
        let config_dir = TestDirectory::new("names-config");
        let store = open_config(&config_dir);

        let error = create_catalog(
            CreateCatalogInput {
                name: "../evil".into(),
                parent_directory: parent.path().to_path_buf(),
            },
            &store,
            |_| Ok(()),
        )
        .unwrap_err();
        assert_eq!(error.code, "invalid_name");

        create_catalog(
            CreateCatalogInput {
                name: "Taken".into(),
                parent_directory: parent.path().to_path_buf(),
            },
            &store,
            |_| Ok(()),
        )
        .unwrap();

        let collision = create_catalog(
            CreateCatalogInput {
                name: "Taken".into(),
                parent_directory: parent.path().to_path_buf(),
            },
            &store,
            |_| Ok(()),
        )
        .unwrap_err();
        assert_eq!(collision.code, "catalog_exists");
    }

    #[test]
    fn rolls_back_catalog_when_authorize_fails() {
        let parent = TestDirectory::new("auth-fail");
        let config_dir = TestDirectory::new("auth-fail-config");
        let store = open_config(&config_dir);

        let error = create_catalog(
            CreateCatalogInput {
                name: "Doomed".into(),
                parent_directory: parent.path().to_path_buf(),
            },
            &store,
            |_| {
                Err(CatalogError::new(
                    "authorize_failed",
                    "Asset scope could not be granted.",
                ))
            },
        )
        .unwrap_err();

        assert_eq!(error.code, "authorize_failed");
        assert!(!parent.path().join("Doomed").exists());
        assert_eq!(store.get().unwrap().catalog, None);
    }

    #[test]
    fn catalog_state_setup_ready_and_unavailable() {
        let pictures = TestDirectory::new("pictures");
        let config = AppConfig::default();
        match get_catalog_state(&config, Some(pictures.path().to_path_buf())).unwrap() {
            CatalogState::SetupRequired {
                pictures_directory: Some(path),
            } => assert_eq!(path, pictures.path()),
            other => panic!("expected setup required, got {other:?}"),
        }

        let (_parent, _config_dir, catalog_path, store) = create_ready_catalog("Ready Catalog");
        let config = store.get().unwrap();
        match get_catalog_state(&config, None).unwrap() {
            CatalogState::Ready { catalog } => {
                assert_eq!(catalog.name, "Ready Catalog");
                assert!(catalog.shoots.is_empty());
            }
            other => panic!("expected ready, got {other:?}"),
        }

        // Missing folder → unavailable.
        std::fs::remove_dir_all(&catalog_path).unwrap();
        match get_catalog_state(&config, None).unwrap() {
            CatalogState::Unavailable { reason, .. } => {
                assert!(reason.contains("missing"));
            }
            other => panic!("expected unavailable, got {other:?}"),
        }
    }

    #[test]
    fn unavailable_when_manifest_name_mismatches_config() {
        let (_parent, _config_dir, catalog_path, store) = create_ready_catalog("Configured");
        let mut manifest = manifest::read_manifest(&catalog_path).unwrap();
        manifest.name = "Different".into();
        manifest::write_manifest(&catalog_path, &manifest).unwrap();

        let config = store.get().unwrap();
        match get_catalog_state(&config, None).unwrap() {
            CatalogState::Unavailable { reason, .. } => {
                assert!(reason.contains("does not match"));
            }
            other => panic!("expected unavailable, got {other:?}"),
        }
    }

    #[test]
    fn import_preserves_nested_structure_and_bytes() {
        let (_parent, _config_dir, catalog_path, _store) = create_ready_catalog("Import Catalog");
        let source = TestDirectory::new("source");
        write_png(&source.path().join("day1/a.png"), 4, 6);
        write_jpeg(&source.path().join("day2/b.JPG"), 8, 10);
        // Unsupported + corrupt
        std::fs::write(source.path().join("notes.txt"), b"hello").unwrap();
        std::fs::write(source.path().join("broken.jpg"), b"not-a-jpeg").unwrap();
        std::fs::create_dir_all(source.path().join("day2/nested")).unwrap();
        std::fs::write(source.path().join("day2/nested/skip.tiff"), b"tiff").unwrap();
        // A folder whose only supported-looking photograph fails to decode. It must
        // not survive into the Catalog as an empty directory.
        std::fs::create_dir_all(source.path().join("day3")).unwrap();
        std::fs::write(source.path().join("day3/corrupt.png"), b"not-a-png").unwrap();

        let original_a = std::fs::read(source.path().join("day1/a.png")).unwrap();
        let cancel = AtomicBool::new(false);
        let mut phases = Vec::new();
        let result = import_shoot(
            &catalog_path,
            source.path(),
            "Spring Shoot",
            |progress| phases.push(progress.phase.clone()),
            &cancel,
        )
        .expect("import should succeed");

        assert!(phases.iter().any(|p| p == "scanning"));
        assert!(phases.iter().any(|p| p == "preparing"));
        assert!(phases.iter().any(|p| p == "finalizing"));

        assert_eq!(result.shoot.summary.photograph_count, 2);
        assert!(result.skipped_file_count >= 2);

        let copied = catalog_path.join("Spring Shoot/day1/a.png");
        assert!(copied.exists());
        assert_eq!(std::fs::read(&copied).unwrap(), original_a);

        // Folders that contributed no photograph leave no trace in the Catalog.
        assert!(!catalog_path.join("Spring Shoot/day3").exists());
        assert!(!catalog_path.join("Spring Shoot/day2/nested").exists());

        let details = get_shoot(&catalog_path, "Spring Shoot").unwrap();
        let paths: Vec<_> = details
            .photographs
            .iter()
            .map(|p| p.relative_path.as_str())
            .collect();
        assert_eq!(paths, vec!["day1/a.png", "day2/b.JPG"]);

        let a = &details.photographs[0];
        assert_eq!((a.width, a.height), (4, 6));
        assert!(a.thumbnail_absolute_path.exists());
        assert!(a
            .thumbnail_relative_path
            .starts_with(".photart/thumbnails/Spring Shoot/"));

        // Source untouched.
        assert!(source.path().join("day1/a.png").exists());
        assert_eq!(
            std::fs::read(source.path().join("day1/a.png")).unwrap(),
            original_a
        );
    }

    #[test]
    fn case_insensitive_extension_discovery() {
        let (_parent, _config_dir, catalog_path, _store) = create_ready_catalog("Ext Catalog");
        let source = TestDirectory::new("ext-source");
        write_jpeg(&source.path().join("one.JpEg"), 3, 3);
        write_png(&source.path().join("two.PNG"), 3, 3);
        write_jpeg(&source.path().join("three.jPeG"), 3, 3);
        assert!(crate::image_processing::is_supported_extension(Path::new(
            "shot.WebP"
        )));

        let cancel = AtomicBool::new(false);
        let result =
            import_shoot(&catalog_path, source.path(), "Ext Shoot", |_| {}, &cancel).unwrap();
        assert_eq!(result.shoot.summary.photograph_count, 3);
    }

    #[test]
    fn skips_symlinks_and_non_files() {
        let (_parent, _config_dir, catalog_path, _store) = create_ready_catalog("Sym Catalog");
        let source = TestDirectory::new("sym-source");
        write_png(&source.path().join("real.png"), 2, 2);

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let linked_file = source.path().join("link.png");
            symlink(source.path().join("real.png"), &linked_file).unwrap();
            let linked_dir = source.path().join("subdir_link");
            let other = TestDirectory::new("sym-other");
            write_png(&other.path().join("hidden.png"), 2, 2);
            symlink(other.path(), &linked_dir).unwrap();
        }

        let cancel = AtomicBool::new(false);
        let result =
            import_shoot(&catalog_path, source.path(), "Sym Shoot", |_| {}, &cancel).unwrap();
        assert_eq!(result.shoot.summary.photograph_count, 1);
        assert_eq!(result.shoot.photographs[0].relative_path, "real.png");
    }

    #[test]
    fn rejects_zero_supported_photographs() {
        let (_parent, _config_dir, catalog_path, _store) = create_ready_catalog("Empty Catalog");
        let source = TestDirectory::new("empty-source");
        std::fs::write(source.path().join("readme.txt"), b"hi").unwrap();
        std::fs::write(source.path().join("bad.jpg"), b"nope").unwrap();

        let cancel = AtomicBool::new(false);
        let error =
            import_shoot(&catalog_path, source.path(), "Empty Shoot", |_| {}, &cancel).unwrap_err();
        assert_eq!(error.code, "no_photographs");
        assert!(!catalog_path.join("Empty Shoot").exists());
        let manifest = manifest::read_manifest(&catalog_path).unwrap();
        assert!(manifest.shoots.is_empty());
    }

    #[test]
    fn duplicate_shoot_rejection_leaves_existing_unchanged() {
        let (_parent, _config_dir, catalog_path, _store) = create_ready_catalog("Dup Catalog");
        let source = TestDirectory::new("dup-source");
        write_png(&source.path().join("a.png"), 2, 2);
        let cancel = AtomicBool::new(false);
        import_shoot(&catalog_path, source.path(), "Same", |_| {}, &cancel).unwrap();
        let before = std::fs::read_to_string(paths::manifest_path(&catalog_path)).unwrap();

        let error =
            import_shoot(&catalog_path, source.path(), "Same", |_| {}, &cancel).unwrap_err();
        assert_eq!(error.code, "shoot_exists");
        let after = std::fs::read_to_string(paths::manifest_path(&catalog_path)).unwrap();
        assert_eq!(before, after);
    }

    #[test]
    fn rejects_source_catalog_overlap() {
        let (_parent, _config_dir, catalog_path, _store) = create_ready_catalog("Overlap Catalog");
        write_png(&catalog_path.join("inside.png"), 2, 2);
        let cancel = AtomicBool::new(false);

        let error =
            import_shoot(&catalog_path, &catalog_path, "Overlap", |_| {}, &cancel).unwrap_err();
        assert_eq!(error.code, "source_catalog_overlap");
    }

    #[test]
    fn thumbnail_paths_unique_for_nested_same_filenames() {
        let (_parent, _config_dir, catalog_path, _store) = create_ready_catalog("Thumb Catalog");
        let source = TestDirectory::new("thumb-source");
        write_png(&source.path().join("a/photo.png"), 2, 2);
        write_png(&source.path().join("b/photo.png"), 3, 3);

        let cancel = AtomicBool::new(false);
        let result =
            import_shoot(&catalog_path, source.path(), "Thumb Shoot", |_| {}, &cancel).unwrap();

        let thumbs: Vec<_> = result
            .shoot
            .photographs
            .iter()
            .map(|p| p.thumbnail_relative_path.clone())
            .collect();
        assert_eq!(thumbs.len(), 2);
        assert_ne!(thumbs[0], thumbs[1]);
        for thumb in &thumbs {
            assert!(catalog_path.join(thumb).exists());
        }
    }

    #[test]
    fn shoots_are_newest_first_and_photos_sorted() {
        let (_parent, _config_dir, catalog_path, store) = create_ready_catalog("Order Catalog");
        let source_a = TestDirectory::new("order-a");
        let source_b = TestDirectory::new("order-b");
        write_png(&source_a.path().join("z.png"), 2, 2);
        write_png(&source_a.path().join("a.png"), 2, 2);
        write_png(&source_b.path().join("only.png"), 2, 2);

        let cancel = AtomicBool::new(false);
        import_shoot(&catalog_path, source_a.path(), "First", |_| {}, &cancel).unwrap();
        thread::sleep(Duration::from_millis(1100));
        import_shoot(&catalog_path, source_b.path(), "Second", |_| {}, &cancel).unwrap();

        let config = store.get().unwrap();
        let CatalogState::Ready { catalog } = get_catalog_state(&config, None).unwrap() else {
            panic!("expected ready");
        };
        assert_eq!(
            catalog
                .shoots
                .iter()
                .map(|s| s.name.as_str())
                .collect::<Vec<_>>(),
            vec!["Second", "First"]
        );

        let first = get_shoot(&catalog_path, "First").unwrap();
        assert_eq!(
            first
                .photographs
                .iter()
                .map(|p| p.relative_path.as_str())
                .collect::<Vec<_>>(),
            vec!["a.png", "z.png"]
        );
    }

    #[test]
    fn cancellation_during_prepare_rolls_back_completely() {
        let (_parent, _config_dir, catalog_path, _store) = create_ready_catalog("Cancel Catalog");
        let source = TestDirectory::new("cancel-source");
        write_png(&source.path().join("a.png"), 2, 2);
        write_png(&source.path().join("b.png"), 2, 2);
        let original = std::fs::read(source.path().join("a.png")).unwrap();

        let cancel = AtomicBool::new(false);
        let faults = ImportFaults {
            fail_after_prepared: Some(1),
            ..ImportFaults::default()
        };
        let error = import_shoot_inner(
            &catalog_path,
            source.path(),
            "Cancel Shoot",
            &mut |_| {},
            &cancel,
            &faults,
        )
        .unwrap_err();
        assert_eq!(error.code, "import_fault");

        assert!(!catalog_path.join("Cancel Shoot").exists());
        assert!(manifest::read_manifest(&catalog_path)
            .unwrap()
            .shoots
            .is_empty());
        assert!(!paths::shoot_thumbnails_dir(&catalog_path, "Cancel Shoot").exists());
        // Staging cleaned.
        let imports = paths::imports_dir(&catalog_path);
        let leftover: Vec<_> = std::fs::read_dir(&imports)
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert!(leftover.is_empty());
        assert_eq!(
            std::fs::read(source.path().join("a.png")).unwrap(),
            original
        );
    }

    #[test]
    fn cancellation_during_copy_rolls_back() {
        let (_parent, _config_dir, catalog_path, _store) = create_ready_catalog("CopyCancel");
        let source = TestDirectory::new("copy-cancel-source");
        write_png(&source.path().join("a.png"), 2, 2);
        write_png(&source.path().join("b.png"), 2, 2);
        write_png(&source.path().join("c.png"), 2, 2);

        let cancel = AtomicBool::new(false);
        let faults = ImportFaults {
            fail_after_copied: Some(1),
            ..ImportFaults::default()
        };
        let error = import_shoot_inner(
            &catalog_path,
            source.path(),
            "Copy Shoot",
            &mut |_| {},
            &cancel,
            &faults,
        )
        .unwrap_err();
        assert_eq!(error.code, "import_fault");
        assert!(!catalog_path.join("Copy Shoot").exists());
        assert!(manifest::read_manifest(&catalog_path)
            .unwrap()
            .shoots
            .is_empty());
    }

    #[test]
    fn failure_before_manifest_leaves_no_indexed_shoot() {
        let (_parent, _config_dir, catalog_path, _store) = create_ready_catalog("ManifestFail");
        let source = TestDirectory::new("manifest-fail-source");
        write_png(&source.path().join("a.png"), 2, 2);
        let original = std::fs::read(source.path().join("a.png")).unwrap();

        let cancel = AtomicBool::new(false);
        let faults = ImportFaults {
            fail_before_manifest: true,
            ..ImportFaults::default()
        };
        let error = import_shoot_inner(
            &catalog_path,
            source.path(),
            "Ghost Shoot",
            &mut |_| {},
            &cancel,
            &faults,
        )
        .unwrap_err();
        assert_eq!(error.code, "import_fault");
        assert!(!catalog_path.join("Ghost Shoot").exists());
        assert!(!paths::shoot_thumbnails_dir(&catalog_path, "Ghost Shoot").exists());
        assert!(manifest::read_manifest(&catalog_path)
            .unwrap()
            .shoots
            .is_empty());
        assert_eq!(
            std::fs::read(source.path().join("a.png")).unwrap(),
            original
        );
    }

    #[test]
    fn successful_import_updates_manifest_only_after_files_exist() {
        let (_parent, _config_dir, catalog_path, _store) = create_ready_catalog("Commit Catalog");
        let source = TestDirectory::new("commit-source");
        write_png(&source.path().join("nested/photo.png"), 5, 7);

        let cancel = AtomicBool::new(false);
        let result = import_shoot(
            &catalog_path,
            source.path(),
            "Commit Shoot",
            |_| {},
            &cancel,
        )
        .unwrap();

        assert!(catalog_path.join("Commit Shoot/nested/photo.png").exists());
        let thumb = &result.shoot.photographs[0].thumbnail_absolute_path;
        assert!(thumb.exists());

        let manifest = manifest::read_manifest(&catalog_path).unwrap();
        assert_eq!(manifest.shoots.len(), 1);
        assert_eq!(manifest.shoots[0].name, "Commit Shoot");
        assert_eq!(manifest.shoots[0].photograph_count, 1);
        assert_eq!(result.shoot.photographs[0].width, 5);
        assert_eq!(result.shoot.photographs[0].height, 7);
    }

    #[test]
    fn path_authorization_only_allows_catalog_descendants() {
        let (_parent, _config_dir, catalog_path, _store) = create_ready_catalog("Auth Catalog");
        let inside = catalog_path.join(".photart/thumbnails/x.jpg");
        std::fs::create_dir_all(inside.parent().unwrap()).unwrap();
        std::fs::write(&inside, b"jpg").unwrap();

        let outside = TestDirectory::new("outside");
        let outside_file = outside.path().join("x.jpg");
        std::fs::write(&outside_file, b"jpg").unwrap();

        assert!(authorize_catalog_path(&catalog_path, &inside).is_ok());
        assert!(is_path_inside_catalog(&catalog_path, &inside));
        assert_eq!(
            authorize_catalog_path(&catalog_path, &outside_file)
                .unwrap_err()
                .code,
            "unauthorized_path"
        );
        assert!(!is_path_inside_catalog(&catalog_path, &outside_file));
    }

    #[test]
    fn import_coordinator_enforces_single_active_import() {
        let coordinator = ImportCoordinator::new();
        let first = coordinator.begin("import-a".into()).unwrap();
        assert_eq!(
            coordinator.begin("import-b".into()).unwrap_err().code,
            "import_busy"
        );
        coordinator.cancel(&first.id).unwrap();
        assert!(first.cancel.load(Ordering::SeqCst));
        coordinator.cancel(&first.id).unwrap();
        coordinator.finish(&first.id).unwrap();
        let second = coordinator.begin("import-b".into()).unwrap();
        coordinator.finish(&second.id).unwrap();
        coordinator.cancel(&second.id).unwrap();
    }

    #[test]
    fn atomic_cancel_flag_aborts_import() {
        let (_parent, _config_dir, catalog_path, _store) = create_ready_catalog("Flag Catalog");
        let source = TestDirectory::new("flag-source");
        for i in 0..20 {
            write_png(&source.path().join(format!("p{i}.png")), 2, 2);
        }

        let cancel = Arc::new(AtomicBool::new(false));
        let cancel_flag = Arc::clone(&cancel);
        cancel_flag.store(true, Ordering::SeqCst);

        let error =
            import_shoot(&catalog_path, source.path(), "Flag Shoot", |_| {}, &cancel).unwrap_err();
        assert_eq!(error.code, "import_cancelled");
        assert!(manifest::read_manifest(&catalog_path)
            .unwrap()
            .shoots
            .is_empty());
    }

    #[test]
    fn empty_catalog_manifest_round_trips() {
        let catalog = TestDirectory::new("roundtrip");
        std::fs::create_dir_all(paths::photart_dir(catalog.path())).unwrap();
        let manifest = CatalogManifest::new("Roundtrip".into());
        manifest::write_manifest(catalog.path(), &manifest).unwrap();
        let loaded = manifest::read_manifest(catalog.path()).unwrap();
        assert_eq!(loaded, manifest);
        assert!(!paths::manifest_path(catalog.path())
            .with_file_name(".catalog.json.tmp")
            .exists());
    }
}
