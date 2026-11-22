use std::fs;
use std::path::Path;

use crate::app_config::{catalog_pointer_from_config_file, CatalogPointer};

use super::manifest;
use super::paths;

/// Pick a Catalog for a fresh dev config.
///
/// An explicit pointer in a legacy checkout config wins when that folder is still
/// a readable Catalog. Otherwise, adopt the Catalog only when Pictures contains
/// exactly one. Several catalogs stay unresolved so setup can ask.
pub fn resolve_dev_catalog_seed(
    legacy_config: Option<&Path>,
    pictures_directory: Option<&Path>,
) -> Option<CatalogPointer> {
    if let Some(path) = legacy_config {
        if let Some(pointer) = catalog_pointer_from_config_file(path) {
            if let Some(verified) = pointer_if_catalog(&pointer.path) {
                return Some(verified);
            }
        }
    }

    pictures_directory.and_then(discover_single_catalog)
}

fn discover_single_catalog(pictures_directory: &Path) -> Option<CatalogPointer> {
    if !pictures_directory.is_dir() {
        return None;
    }

    let entries = fs::read_dir(pictures_directory).ok()?;
    let mut found = None;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(pointer) = pointer_if_catalog(&path) else {
            continue;
        };
        if found.is_some() {
            return None;
        }
        found = Some(pointer);
    }

    found
}

fn pointer_if_catalog(path: &Path) -> Option<CatalogPointer> {
    if !paths::manifest_path(path).is_file() {
        return None;
    }

    let manifest = manifest::read_manifest(path).ok()?;
    let absolute = paths::canonicalize_existing(path).unwrap_or_else(|_| path.to_path_buf());
    Some(CatalogPointer {
        name: manifest.name,
        path: absolute,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;
    use crate::app_config::AppConfigStore;
    use crate::catalog::manifest::{self, CatalogManifest};

    static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct TestDirectory(std::path::PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let sequence = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "photart-discover-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir_all(&path).expect("test directory should be created");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn write_catalog(parent: &Path, name: &str) -> std::path::PathBuf {
        let catalog_path = parent.join(name);
        fs::create_dir(&catalog_path).expect("catalog directory should be created");
        manifest::write_manifest(&catalog_path, &CatalogManifest::new(name.to_string()))
            .expect("manifest should be written");
        catalog_path
    }

    #[test]
    fn pictures_without_a_catalog_stays_unset() {
        let pictures = TestDirectory::new();
        fs::create_dir(pictures.path().join("Loose Photos")).expect("dir");

        assert_eq!(resolve_dev_catalog_seed(None, Some(pictures.path())), None);
    }

    #[test]
    fn a_single_pictures_catalog_is_adopted() {
        let pictures = TestDirectory::new();
        let catalog_path = write_catalog(pictures.path(), "Photart Catalog");
        fs::create_dir(pictures.path().join("Photo Booth Library")).expect("dir");

        let pointer = resolve_dev_catalog_seed(None, Some(pictures.path()))
            .expect("the only catalog should be adopted");

        assert_eq!(pointer.name, "Photart Catalog");
        assert_eq!(
            pointer.path,
            paths::canonicalize_existing(&catalog_path).expect("catalog path")
        );
    }

    #[test]
    fn several_pictures_catalogs_are_left_for_setup() {
        let pictures = TestDirectory::new();
        write_catalog(pictures.path(), "Studio");
        write_catalog(pictures.path(), "Travel");

        assert_eq!(resolve_dev_catalog_seed(None, Some(pictures.path())), None);
    }

    #[test]
    fn a_legacy_checkout_pointer_wins_over_pictures() {
        let root = TestDirectory::new();
        let legacy_catalog = write_catalog(root.path(), "From Checkout");
        let pictures = TestDirectory::new();
        write_catalog(pictures.path(), "From Pictures");

        let config_path = root.path().join("photart.config.json");
        let store = AppConfigStore::open(config_path.clone()).expect("config opens");
        store
            .set_catalog(Some(CatalogPointer {
                name: "From Checkout".to_string(),
                path: legacy_catalog.clone(),
            }))
            .expect("pointer should persist");

        let pointer = resolve_dev_catalog_seed(Some(&config_path), Some(pictures.path()))
            .expect("legacy pointer should be used");

        assert_eq!(pointer.name, "From Checkout");
        assert_eq!(
            pointer.path,
            paths::canonicalize_existing(&legacy_catalog).expect("catalog path")
        );
    }

    #[test]
    fn a_stale_legacy_pointer_falls_through_to_pictures() {
        let root = TestDirectory::new();
        let pictures = TestDirectory::new();
        let catalog_path = write_catalog(pictures.path(), "Photart Catalog");

        let config_path = root.path().join("photart.config.json");
        let store = AppConfigStore::open(config_path.clone()).expect("config opens");
        store
            .set_catalog(Some(CatalogPointer {
                name: "Missing".to_string(),
                path: root.path().join("gone"),
            }))
            .expect("pointer should persist");

        let pointer = resolve_dev_catalog_seed(Some(&config_path), Some(pictures.path()))
            .expect("pictures catalog should be adopted");

        assert_eq!(pointer.name, "Photart Catalog");
        assert_eq!(
            pointer.path,
            paths::canonicalize_existing(&catalog_path).expect("catalog path")
        );
    }

    #[test]
    fn an_empty_legacy_config_falls_through_to_pictures() {
        let root = TestDirectory::new();
        let pictures = TestDirectory::new();
        write_catalog(pictures.path(), "Photart Catalog");

        let config_path = root.path().join("photart.config.json");
        AppConfigStore::open(config_path.clone()).expect("config opens");

        let pointer = resolve_dev_catalog_seed(Some(&config_path), Some(pictures.path()))
            .expect("pictures catalog should be adopted");
        assert_eq!(pointer.name, "Photart Catalog");
    }
}
