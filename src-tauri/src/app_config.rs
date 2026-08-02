use std::fmt;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Runtime};

const CONFIG_FILE_NAME: &str = "config.json";
const DEVELOPMENT_CONFIG_FILE_NAME: &str = "photart.config.json";
const CURRENT_SCHEMA_VERSION: u32 = 2;
const LEGACY_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogPointer {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub schema_version: u32,
    pub catalog: Option<CatalogPointer>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            catalog: None,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyAppConfigV1 {
    schema_version: u32,
    catalogue_path: Option<PathBuf>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfigError {
    pub code: String,
    pub message: String,
}

impl AppConfigError {
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
        }
    }
}

impl fmt::Display for AppConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for AppConfigError {}

pub struct AppConfigStore {
    path: PathBuf,
    config: RwLock<AppConfig>,
}

impl AppConfigStore {
    pub fn open(path: PathBuf) -> Result<Self, AppConfigError> {
        let config = if path.exists() {
            let (config, migrated) = read_config(&path)?;
            if migrated {
                write_config(&path, &config)?;
            }
            config
        } else {
            let config = AppConfig::default();
            write_config(&path, &config)?;
            config
        };

        Ok(Self {
            path,
            config: RwLock::new(config),
        })
    }

    pub fn get(&self) -> Result<AppConfig, AppConfigError> {
        self.config
            .read()
            .map(|config| config.clone())
            .map_err(|_| {
                AppConfigError::new("config_lock", "The app configuration is unavailable.")
            })
    }

    pub fn set_catalog(
        &self,
        catalog: Option<CatalogPointer>,
    ) -> Result<AppConfig, AppConfigError> {
        let mut current = self.config.write().map_err(|_| {
            AppConfigError::new("config_lock", "The app configuration is unavailable.")
        })?;
        let mut next = current.clone();
        next.schema_version = CURRENT_SCHEMA_VERSION;
        next.catalog = catalog;
        write_config(&self.path, &next)?;
        *current = next.clone();
        Ok(next)
    }
}

pub fn config_path<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, AppConfigError> {
    if cfg!(debug_assertions) {
        let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .ok_or_else(|| {
                AppConfigError::new(
                    "config_path",
                    "The development project root could not be resolved.",
                )
            })?;
        Ok(project_root.join(DEVELOPMENT_CONFIG_FILE_NAME))
    } else {
        app.path()
            .app_config_dir()
            .map(|directory| directory.join(CONFIG_FILE_NAME))
            .map_err(|error| {
                AppConfigError::new(
                    "config_path",
                    format!("The operating system config directory could not be resolved: {error}"),
                )
            })
    }
}

fn migrate_from_v1(legacy: LegacyAppConfigV1) -> Result<AppConfig, AppConfigError> {
    if legacy.schema_version != LEGACY_SCHEMA_VERSION {
        return Err(AppConfigError::new(
            "config_version",
            format!(
                "App configuration uses unsupported schema version {}.",
                legacy.schema_version
            ),
        ));
    }

    let catalog = legacy.catalogue_path.map(|path| {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .filter(|name| !name.is_empty())
            .unwrap_or("Catalog")
            .to_string();
        CatalogPointer { name, path }
    });

    Ok(AppConfig {
        schema_version: CURRENT_SCHEMA_VERSION,
        catalog,
    })
}

fn read_config(path: &Path) -> Result<(AppConfig, bool), AppConfigError> {
    let contents = fs::read_to_string(path).map_err(|error| {
        AppConfigError::new(
            "config_read",
            format!(
                "Could not read app configuration at {}: {error}",
                path.display()
            ),
        )
    })?;

    let value: serde_json::Value = serde_json::from_str(&contents).map_err(|error| {
        AppConfigError::new(
            "config_parse",
            format!(
                "Could not parse app configuration at {}: {error}",
                path.display()
            ),
        )
    })?;

    let schema_version = value
        .get("schemaVersion")
        .and_then(|version| version.as_u64())
        .ok_or_else(|| {
            AppConfigError::new(
                "config_parse",
                format!(
                    "Could not parse app configuration at {}: missing schemaVersion.",
                    path.display()
                ),
            )
        })?;

    match schema_version {
        version if version == u64::from(CURRENT_SCHEMA_VERSION) => {
            let config = serde_json::from_value(value).map_err(|error| {
                AppConfigError::new(
                    "config_parse",
                    format!(
                        "Could not parse app configuration at {}: {error}",
                        path.display()
                    ),
                )
            })?;
            Ok((config, false))
        }
        version if version == u64::from(LEGACY_SCHEMA_VERSION) => {
            let legacy: LegacyAppConfigV1 = serde_json::from_value(value).map_err(|error| {
                AppConfigError::new(
                    "config_parse",
                    format!(
                        "Could not parse app configuration at {}: {error}",
                        path.display()
                    ),
                )
            })?;
            Ok((migrate_from_v1(legacy)?, true))
        }
        version => Err(AppConfigError::new(
            "config_version",
            format!(
                "App configuration at {} uses unsupported schema version {version}.",
                path.display()
            ),
        )),
    }
}

fn write_config(path: &Path, config: &AppConfig) -> Result<(), AppConfigError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            AppConfigError::new(
                "config_directory",
                format!(
                    "Could not create app configuration directory at {}: {error}",
                    parent.display()
                ),
            )
        })?;
    }

    let contents = serde_json::to_string_pretty(config).map_err(|error| {
        AppConfigError::new(
            "config_serialize",
            format!("Could not serialize app configuration: {error}"),
        )
    })?;
    let contents = format!("{contents}\n");

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| {
            AppConfigError::new(
                "config_write",
                format!(
                    "Could not write app configuration at {}: invalid file name.",
                    path.display()
                ),
            )
        })?;
    let temporary_path = path.with_file_name(format!(".{file_name}.tmp"));

    {
        let mut temporary = fs::File::create(&temporary_path).map_err(|error| {
            AppConfigError::new(
                "config_write",
                format!(
                    "Could not write app configuration at {}: {error}",
                    path.display()
                ),
            )
        })?;
        temporary.write_all(contents.as_bytes()).map_err(|error| {
            AppConfigError::new(
                "config_write",
                format!(
                    "Could not write app configuration at {}: {error}",
                    path.display()
                ),
            )
        })?;
        temporary.sync_all().map_err(|error| {
            AppConfigError::new(
                "config_write",
                format!(
                    "Could not write app configuration at {}: {error}",
                    path.display()
                ),
            )
        })?;
    }

    fs::rename(&temporary_path, path).map_err(|error| {
        let _ = fs::remove_file(&temporary_path);
        AppConfigError::new(
            "config_write",
            format!(
                "Could not write app configuration at {}: {error}",
                path.display()
            ),
        )
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;

    static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let sequence = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "photart-app-config-{}-{sequence}",
                std::process::id()
            ));
            fs::create_dir_all(&path).expect("test config directory should be created");
            Self(path)
        }

        fn config_path(&self) -> PathBuf {
            self.0.join(CONFIG_FILE_NAME)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn opening_a_missing_config_creates_defaults() {
        let directory = TestDirectory::new();
        let path = directory.config_path();

        let store = AppConfigStore::open(path.clone()).expect("config should open");

        assert_eq!(
            store.get().expect("config should be readable"),
            AppConfig::default()
        );
        assert!(path.exists());
        assert_eq!(
            read_config(&path).expect("file should contain config").0,
            AppConfig::default()
        );
    }

    #[test]
    fn opening_an_existing_config_loads_it() {
        let directory = TestDirectory::new();
        let path = directory.config_path();
        let expected = AppConfig {
            catalog: Some(CatalogPointer {
                name: "Photart Catalog".to_string(),
                path: PathBuf::from("/photos/Photart Catalog"),
            }),
            ..AppConfig::default()
        };
        write_config(&path, &expected).expect("fixture should be written");

        let store = AppConfigStore::open(path).expect("config should open");

        assert_eq!(store.get().expect("config should be readable"), expected);
    }

    #[test]
    fn setting_the_catalog_persists_it() {
        let directory = TestDirectory::new();
        let path = directory.config_path();
        let store = AppConfigStore::open(path.clone()).expect("config should open");
        let catalog = CatalogPointer {
            name: "Photart Catalog".to_string(),
            path: PathBuf::from("/photos/Photart Catalog"),
        };

        let updated = store
            .set_catalog(Some(catalog.clone()))
            .expect("config should update");

        assert_eq!(updated.catalog, Some(catalog));
        assert_eq!(
            read_config(&path)
                .expect("persisted config should be readable")
                .0,
            updated
        );
    }

    #[test]
    fn opening_invalid_json_does_not_replace_the_file() {
        let directory = TestDirectory::new();
        let path = directory.config_path();
        let invalid_contents = "{ not valid json";
        fs::write(&path, invalid_contents).expect("fixture should be written");

        let error = AppConfigStore::open(path.clone())
            .err()
            .expect("invalid config should fail");

        assert_eq!(error.code, "config_parse");
        assert_eq!(
            fs::read_to_string(path).expect("fixture should remain readable"),
            invalid_contents
        );
    }

    #[test]
    fn migrates_v1_null_catalogue_path_to_null_catalog() {
        let directory = TestDirectory::new();
        let path = directory.config_path();
        fs::write(
            &path,
            "{\n  \"schemaVersion\": 1,\n  \"cataloguePath\": null\n}\n",
        )
        .expect("fixture should be written");

        let store = AppConfigStore::open(path.clone()).expect("config should open");
        let config = store.get().expect("config should be readable");

        assert_eq!(config.schema_version, CURRENT_SCHEMA_VERSION);
        assert_eq!(config.catalog, None);
        assert_eq!(
            read_config(&path)
                .expect("migrated config should be readable")
                .0,
            config
        );
    }

    #[test]
    fn migrates_v1_catalogue_path_inferring_name_from_final_segment() {
        let directory = TestDirectory::new();
        let path = directory.config_path();
        fs::write(
            &path,
            "{\n  \"schemaVersion\": 1,\n  \"cataloguePath\": \"/Users/me/Pictures/Old Catalogue\"\n}\n",
        )
        .expect("fixture should be written");

        let store = AppConfigStore::open(path).expect("config should open");
        let config = store.get().expect("config should be readable");

        assert_eq!(
            config.catalog,
            Some(CatalogPointer {
                name: "Old Catalogue".to_string(),
                path: PathBuf::from("/Users/me/Pictures/Old Catalogue"),
            })
        );
    }

    #[test]
    fn unsupported_schema_version_preserves_file() {
        let directory = TestDirectory::new();
        let path = directory.config_path();
        let contents = "{\n  \"schemaVersion\": 99,\n  \"catalog\": null\n}\n";
        fs::write(&path, contents).expect("fixture should be written");

        let error = AppConfigStore::open(path.clone())
            .err()
            .expect("unsupported version should fail");

        assert_eq!(error.code, "config_version");
        assert_eq!(
            fs::read_to_string(path).expect("fixture should remain readable"),
            contents
        );
    }

    #[test]
    fn write_config_uses_atomic_sibling_temporary_file() {
        let directory = TestDirectory::new();
        let path = directory.config_path();
        let config = AppConfig::default();
        write_config(&path, &config).expect("config should write");
        assert!(path.exists());
        assert!(!path.with_file_name(".config.json.tmp").exists());
    }
}
