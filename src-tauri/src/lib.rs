mod app_config;
mod catalog;
mod image_processing;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use app_config::{AppConfigError, AppConfigStore};
use catalog::{
    create_catalog as create_catalog_on_disk, get_catalog_state as resolve_catalog_state,
    get_shoot as resolve_shoot, import_shoot as run_import_shoot,
    purge_shoot as purge_catalog_shoot, rename_shoot as rename_catalog_shoot, CatalogError,
    CatalogSnapshot, CatalogState, CreateCatalogInput, ImportCoordinator, ImportProgress,
    ImportShootResult, ShootDetails,
};
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, Runtime, State};

enum ConfigStatus {
    Ready,
    Failed(AppConfigError),
}

#[tauri::command]
fn get_catalog_state(app: AppHandle) -> Result<CatalogState, CatalogError> {
    let store = require_config_store(&app)?;
    let app_config = store.get().map_err(config_error_to_catalog)?;
    let pictures_directory = app.path().picture_dir().ok();
    let state = resolve_catalog_state(&app_config, pictures_directory)?;

    if let CatalogState::Ready { catalog } = &state {
        authorize_catalog_scope(&app, &catalog.path)?;
    }

    Ok(state)
}

#[tauri::command]
fn create_catalog(
    app: AppHandle,
    name: String,
    parent_directory: PathBuf,
) -> Result<CatalogSnapshot, CatalogError> {
    let store = require_config_store(&app)?;
    create_catalog_on_disk(
        CreateCatalogInput {
            name,
            parent_directory,
        },
        &store,
        |catalog_path| authorize_catalog_scope(&app, catalog_path),
    )
}

#[tauri::command]
fn get_shoot(app: AppHandle, shoot_name: String) -> Result<ShootDetails, CatalogError> {
    let catalog_path = require_ready_catalog_path(&app)?;
    resolve_shoot(&catalog_path, &shoot_name)
}

#[tauri::command]
async fn rename_shoot(
    app: AppHandle,
    shoot_name: String,
    new_name: String,
) -> Result<ShootDetails, CatalogError> {
    let catalog_path = require_ready_catalog_path(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        rename_catalog_shoot(&catalog_path, &shoot_name, &new_name)
    })
    .await
    .map_err(|error| {
        CatalogError::new(
            "shoot_rename_join",
            format!("Shoot rename could not finish: {error}"),
        )
    })?
}

#[tauri::command]
async fn purge_shoot(app: AppHandle, shoot_name: String) -> Result<(), CatalogError> {
    let catalog_path = require_ready_catalog_path(&app)?;
    tauri::async_runtime::spawn_blocking(move || purge_catalog_shoot(&catalog_path, &shoot_name))
        .await
        .map_err(|error| {
            CatalogError::new(
                "shoot_purge_join",
                format!("Shoot purge could not finish: {error}"),
            )
        })?
}

#[tauri::command]
async fn import_shoot(
    app: AppHandle,
    import_id: String,
    source_path: PathBuf,
    shoot_name: String,
    on_progress: Channel<ImportProgress>,
    imports: State<'_, ImportCoordinator>,
) -> Result<ImportShootResult, CatalogError> {
    let catalog_path = require_ready_catalog_path(&app)?;
    let permit = imports.begin(import_id)?;
    let cancel = std::sync::Arc::clone(&permit.cancel);
    let finish_id = permit.id.clone();

    // Run filesystem work off the async runtime so Channel progress can reach the UI.
    let result = tauri::async_runtime::spawn_blocking(move || {
        run_import_shoot(
            &catalog_path,
            &source_path,
            &shoot_name,
            |progress| {
                let _ = on_progress.send(progress);
            },
            &cancel,
        )
    })
    .await
    .map_err(|error| {
        CatalogError::new("import_join", format!("Import could not finish: {error}"))
    })?;

    let _ = imports.finish(&finish_id);
    result
}

#[tauri::command]
fn cancel_shoot_import(
    import_id: String,
    imports: State<'_, ImportCoordinator>,
) -> Result<(), CatalogError> {
    imports.cancel(&import_id)
}

fn require_config_store(app: &AppHandle) -> Result<State<'_, AppConfigStore>, CatalogError> {
    let status = app.state::<Mutex<ConfigStatus>>();
    let guard = status
        .lock()
        .map_err(|_| CatalogError::new("config_lock", "The app configuration is unavailable."))?;
    match &*guard {
        ConfigStatus::Ready => {
            drop(guard);
            Ok(app.state::<AppConfigStore>())
        }
        ConfigStatus::Failed(error) => Err(config_error_to_catalog(AppConfigError::new(
            &error.code,
            error.message.clone(),
        ))),
    }
}

fn require_ready_catalog_path(app: &AppHandle) -> Result<PathBuf, CatalogError> {
    let store = require_config_store(app)?;
    let app_config = store.get().map_err(config_error_to_catalog)?;
    let pictures_directory = app.path().picture_dir().ok();
    match resolve_catalog_state(&app_config, pictures_directory)? {
        CatalogState::Ready { catalog } => {
            authorize_catalog_scope(app, &catalog.path)?;
            Ok(catalog.path)
        }
        CatalogState::SetupRequired { .. } => Err(CatalogError::new(
            "catalog_setup_required",
            "Create a Catalog before continuing.",
        )),
        CatalogState::Unavailable { reason, .. } => {
            Err(CatalogError::new("catalog_unavailable", reason))
        }
    }
}

fn authorize_catalog_scope<R: Runtime>(
    app: &AppHandle<R>,
    catalog_path: &Path,
) -> Result<(), CatalogError> {
    app.state::<tauri::scope::Scopes>()
        .allow_directory(catalog_path, true)
        .map_err(|error| {
            CatalogError::new(
                "asset_scope",
                format!(
                    "Could not authorize Catalog asset access at {}: {error}",
                    catalog_path.display()
                ),
            )
        })
}

fn config_error_to_catalog(error: AppConfigError) -> CatalogError {
    CatalogError::new(&error.code, error.message)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let status = match app_config::config_path(app.handle()) {
                Ok(path) => match AppConfigStore::open(path) {
                    Ok(store) => {
                        if let Ok(config) = store.get() {
                            if let Some(catalog) = config.catalog {
                                let _ = app
                                    .state::<tauri::scope::Scopes>()
                                    .allow_directory(&catalog.path, true);
                            }
                        }
                        app.manage(store);
                        ConfigStatus::Ready
                    }
                    Err(error) => ConfigStatus::Failed(error),
                },
                Err(error) => ConfigStatus::Failed(error),
            };
            app.manage(Mutex::new(status));
            app.manage(ImportCoordinator::new());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_catalog_state,
            create_catalog,
            get_shoot,
            rename_shoot,
            purge_shoot,
            import_shoot,
            cancel_shoot_import,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
