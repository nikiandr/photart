use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::error::CatalogError;
use super::paths::{self, validate_relative_path};

pub const MANIFEST_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogManifest {
    pub schema_version: u32,
    pub name: String,
    pub shoots: Vec<ShootRecord>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShootRecord {
    pub name: String,
    pub relative_path: String,
    pub imported_at: String,
    pub photograph_count: u32,
    pub skipped_file_count: u32,
    pub photographs: Vec<PhotographRecord>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotographRecord {
    pub relative_path: String,
    pub file_name: String,
    pub width: u32,
    pub height: u32,
    pub thumbnail_relative_path: String,
}

impl CatalogManifest {
    pub fn new(name: String) -> Self {
        Self {
            schema_version: MANIFEST_SCHEMA_VERSION,
            name,
            shoots: Vec::new(),
        }
    }

    pub fn validate(&self, catalog_path: &Path) -> Result<(), CatalogError> {
        if self.schema_version != MANIFEST_SCHEMA_VERSION {
            return Err(CatalogError::new(
                "manifest_version",
                format!(
                    "Catalog manifest uses unsupported schema version {}.",
                    self.schema_version
                ),
            ));
        }

        if self.name.trim().is_empty() {
            return Err(CatalogError::new(
                "manifest_invalid",
                "Catalog manifest name cannot be empty.",
            ));
        }

        if Path::new(&self.name).is_absolute() {
            return Err(CatalogError::new(
                "manifest_invalid",
                "Catalog manifest name must not be an absolute path.",
            ));
        }

        for shoot in &self.shoots {
            validate_shoot_record(shoot, catalog_path)?;
        }

        Ok(())
    }

    /// Sort shoots newest-first by imported_at; stable by original index on ties.
    pub fn sorted_shoots_newest_first(&self) -> Vec<&ShootRecord> {
        let mut indexed: Vec<(usize, &ShootRecord)> = self.shoots.iter().enumerate().collect();
        indexed.sort_by(|(index_a, shoot_a), (index_b, shoot_b)| {
            shoot_b
                .imported_at
                .cmp(&shoot_a.imported_at)
                .then_with(|| index_a.cmp(index_b))
        });
        indexed.into_iter().map(|(_, shoot)| shoot).collect()
    }
}

fn validate_shoot_record(shoot: &ShootRecord, catalog_path: &Path) -> Result<(), CatalogError> {
    let relative_path = validate_relative_path(&shoot.relative_path)?;
    ensure_resolves_inside(catalog_path, &relative_path)?;

    if shoot.name.trim().is_empty() {
        return Err(CatalogError::new(
            "manifest_invalid",
            "Shoot name cannot be empty.",
        ));
    }

    for photograph in &shoot.photographs {
        let photo_rel = validate_relative_path(&photograph.relative_path)?;
        let joined = format!("{relative_path}/{photo_rel}");
        ensure_resolves_inside(catalog_path, &joined)?;

        let thumb = validate_relative_path(&photograph.thumbnail_relative_path)?;
        ensure_resolves_inside(catalog_path, &thumb)?;

        if photograph.file_name.trim().is_empty() {
            return Err(CatalogError::new(
                "manifest_invalid",
                "Photograph file name cannot be empty.",
            ));
        }
    }

    Ok(())
}

fn ensure_resolves_inside(catalog_path: &Path, relative: &str) -> Result<(), CatalogError> {
    let resolved = catalog_path.join(Path::new(relative));
    // Lexical containment check without requiring the file to exist.
    let catalog_normalized =
        paths::canonicalize_existing(catalog_path).unwrap_or_else(|_| catalog_path.to_path_buf());
    if !paths::is_path_inside_catalog(&catalog_normalized, &resolved)
        && !resolved.starts_with(catalog_path)
    {
        return Err(CatalogError::new(
            "invalid_path",
            format!("Path escapes the Catalog: {relative}"),
        ));
    }

    // Extra guard: after joining, no parent-dir components remain unresolved.
    let mut depth = 0i32;
    for component in Path::new(relative).components() {
        match component {
            std::path::Component::Normal(_) => depth += 1,
            std::path::Component::ParentDir => depth -= 1,
            _ => {}
        }
        if depth < 0 {
            return Err(CatalogError::new(
                "invalid_path",
                format!("Path escapes the Catalog: {relative}"),
            ));
        }
    }

    Ok(())
}

pub fn read_manifest(catalog_path: &Path) -> Result<CatalogManifest, CatalogError> {
    let path = paths::manifest_path(catalog_path);
    let contents = fs::read_to_string(&path).map_err(|error| {
        CatalogError::new(
            "manifest_read",
            format!(
                "Could not read Catalog manifest at {}: {error}",
                path.display()
            ),
        )
    })?;

    let manifest: CatalogManifest = serde_json::from_str(&contents).map_err(|error| {
        CatalogError::new(
            "manifest_parse",
            format!(
                "Could not parse Catalog manifest at {}: {error}",
                path.display()
            ),
        )
    })?;

    manifest.validate(catalog_path)?;
    Ok(manifest)
}

/// Atomically write the manifest via a sibling temporary file.
pub fn write_manifest(catalog_path: &Path, manifest: &CatalogManifest) -> Result<(), CatalogError> {
    manifest.validate(catalog_path)?;

    let path = paths::manifest_path(catalog_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            CatalogError::new(
                "manifest_write",
                format!(
                    "Could not create Catalog metadata directory at {}: {error}",
                    parent.display()
                ),
            )
        })?;
    }

    let contents = serde_json::to_string_pretty(manifest).map_err(|error| {
        CatalogError::new(
            "manifest_serialize",
            format!("Could not serialize Catalog manifest: {error}"),
        )
    })?;
    let contents = format!("{contents}\n");

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            CatalogError::new(
                "manifest_write",
                format!(
                    "Could not write Catalog manifest at {}: invalid file name.",
                    path.display()
                ),
            )
        })?;
    let temporary_path = path.with_file_name(format!(".{file_name}.tmp"));

    {
        let mut temporary = fs::File::create(&temporary_path).map_err(|error| {
            CatalogError::new(
                "manifest_write",
                format!(
                    "Could not write Catalog manifest at {}: {error}",
                    path.display()
                ),
            )
        })?;
        temporary.write_all(contents.as_bytes()).map_err(|error| {
            CatalogError::new(
                "manifest_write",
                format!(
                    "Could not write Catalog manifest at {}: {error}",
                    path.display()
                ),
            )
        })?;
        temporary.sync_all().map_err(|error| {
            CatalogError::new(
                "manifest_write",
                format!(
                    "Could not write Catalog manifest at {}: {error}",
                    path.display()
                ),
            )
        })?;
    }

    fs::rename(&temporary_path, &path).map_err(|error| {
        let _ = fs::remove_file(&temporary_path);
        CatalogError::new(
            "manifest_write",
            format!(
                "Could not write Catalog manifest at {}: {error}",
                path.display()
            ),
        )
    })?;

    Ok(())
}

pub fn absolute_thumbnail_path(catalog_path: &Path, thumbnail_relative: &str) -> PathBuf {
    catalog_path.join(Path::new(thumbnail_relative))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(0);

    fn temp_catalog() -> PathBuf {
        let sequence = NEXT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "photart-manifest-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(paths::photart_dir(&path)).unwrap();
        path
    }

    #[test]
    fn atomic_write_leaves_no_temporary() {
        let catalog = temp_catalog();
        let manifest = CatalogManifest::new("Test Catalog".into());
        write_manifest(&catalog, &manifest).unwrap();

        let path = paths::manifest_path(&catalog);
        assert!(path.exists());
        assert!(!path.with_file_name(".catalog.json.tmp").exists());

        let loaded = read_manifest(&catalog).unwrap();
        assert_eq!(loaded.name, "Test Catalog");
        let _ = fs::remove_dir_all(&catalog);
    }

    #[test]
    fn rejects_unsupported_schema_version() {
        let catalog = temp_catalog();
        let path = paths::manifest_path(&catalog);
        fs::write(
            &path,
            "{\n  \"schemaVersion\": 99,\n  \"name\": \"X\",\n  \"shoots\": []\n}\n",
        )
        .unwrap();

        let error = read_manifest(&catalog).unwrap_err();
        assert_eq!(error.code, "manifest_version");
        let _ = fs::remove_dir_all(&catalog);
    }

    #[test]
    fn rejects_absolute_paths_in_records() {
        let catalog = temp_catalog();
        let mut manifest = CatalogManifest::new("Test".into());
        manifest.shoots.push(ShootRecord {
            name: "Shoot".into(),
            relative_path: "/absolute".into(),
            imported_at: "2024-01-01T00:00:00Z".into(),
            photograph_count: 0,
            skipped_file_count: 0,
            photographs: vec![],
        });
        let error = write_manifest(&catalog, &manifest).unwrap_err();
        assert_eq!(error.code, "invalid_path");
        let _ = fs::remove_dir_all(&catalog);
    }

    #[test]
    fn newest_first_sort_uses_manifest_order_as_tiebreaker() {
        let mut manifest = CatalogManifest::new("Test".into());
        manifest.shoots.push(ShootRecord {
            name: "A".into(),
            relative_path: "A".into(),
            imported_at: "2024-01-01T00:00:00Z".into(),
            photograph_count: 0,
            skipped_file_count: 0,
            photographs: vec![],
        });
        manifest.shoots.push(ShootRecord {
            name: "B".into(),
            relative_path: "B".into(),
            imported_at: "2024-01-02T00:00:00Z".into(),
            photograph_count: 0,
            skipped_file_count: 0,
            photographs: vec![],
        });
        manifest.shoots.push(ShootRecord {
            name: "C".into(),
            relative_path: "C".into(),
            imported_at: "2024-01-02T00:00:00Z".into(),
            photograph_count: 0,
            skipped_file_count: 0,
            photographs: vec![],
        });

        let names: Vec<_> = manifest
            .sorted_shoots_newest_first()
            .iter()
            .map(|s| s.name.as_str())
            .collect();
        // B and C share timestamp; B appears before C in manifest so B wins the tie.
        assert_eq!(names, vec!["B", "C", "A"]);
    }
}
