use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, Runtime};

const CONFIG_FILE_NAME: &str = "config.json";
const DEVELOPMENT_CONFIG_FILE_NAME: &str = "photart.config.json";
const CURRENT_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub schema_version: u32,
    pub catalogue_path: Option<PathBuf>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            catalogue_path: None,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfigError {
    code: String,
    message: String,
}

impl AppConfigError {
    fn new(code: &str, message: impl Into<String>) -> Self {
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
            read_config(&path)?
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

    pub fn set_catalogue_path(
        &self,
        catalogue_path: Option<PathBuf>,
    ) -> Result<AppConfig, AppConfigError> {
        let mut current = self.config.write().map_err(|_| {
            AppConfigError::new("config_lock", "The app configuration is unavailable.")
        })?;
        let mut next = current.clone();
        next.catalogue_path = catalogue_path;
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

fn read_config(path: &Path) -> Result<AppConfig, AppConfigError> {
    let contents = fs::read_to_string(path).map_err(|error| {
        AppConfigError::new(
            "config_read",
            format!(
                "Could not read app configuration at {}: {error}",
                path.display()
            ),
        )
    })?;
    let config: AppConfig = serde_json::from_str(&contents).map_err(|error| {
        AppConfigError::new(
            "config_parse",
            format!(
                "Could not parse app configuration at {}: {error}",
                path.display()
            ),
        )
    })?;

    if config.schema_version == CURRENT_SCHEMA_VERSION {
        Ok(config)
    } else {
        Err(AppConfigError::new(
            "config_version",
            format!(
                "App configuration at {} uses unsupported schema version {}.",
                path.display(),
                config.schema_version
            ),
        ))
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
    fs::write(path, format!("{contents}\n")).map_err(|error| {
        AppConfigError::new(
            "config_write",
            format!(
                "Could not write app configuration at {}: {error}",
                path.display()
            ),
        )
    })
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
            read_config(&path).expect("file should contain config"),
            AppConfig::default()
        );
    }

    #[test]
    fn opening_an_existing_config_loads_it() {
        let directory = TestDirectory::new();
        let path = directory.config_path();
        let expected = AppConfig {
            catalogue_path: Some(PathBuf::from("/photos/catalogue")),
            ..AppConfig::default()
        };
        write_config(&path, &expected).expect("fixture should be written");

        let store = AppConfigStore::open(path).expect("config should open");

        assert_eq!(store.get().expect("config should be readable"), expected);
    }

    #[test]
    fn setting_the_catalogue_path_persists_it() {
        let directory = TestDirectory::new();
        let path = directory.config_path();
        let store = AppConfigStore::open(path.clone()).expect("config should open");
        let catalogue_path = PathBuf::from("/photos/catalogue");

        let updated = store
            .set_catalogue_path(Some(catalogue_path.clone()))
            .expect("config should update");

        assert_eq!(updated.catalogue_path, Some(catalogue_path));
        assert_eq!(
            read_config(&path).expect("persisted config should be readable"),
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
}
