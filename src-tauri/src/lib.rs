mod app_config;
mod catalog;
mod file_timing;
mod image_processing;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use app_config::{AppConfigError, AppConfigStore};
use catalog::{
    create_catalog as create_catalog_on_disk, get_catalog_state as resolve_catalog_state,
    get_shoot as resolve_shoot, import_shoot as run_import_shoot,
    purge_shoot as purge_catalog_shoot, update_shoot as update_catalog_shoot, CatalogError,
    CatalogSnapshot, CatalogState, CreateCatalogInput, ImportCoordinator, ImportProgress,
    ImportShootResult, ShootDetails, ShootMedium, SourceMediumHint,
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
async fn get_photograph_detail(
    app: AppHandle,
    shoot_name: String,
    relative_path: String,
) -> Result<PathBuf, CatalogError> {
    tauri::async_runtime::spawn_blocking(move || {
        let catalog_path = require_ready_catalog_path(&app)?;
        catalog::get_photograph_detail(&catalog_path, &shoot_name, &relative_path)
    })
    .await
    .map_err(|error| {
        CatalogError::new(
            "photograph_detail_join",
            format!("Detail image could not be prepared: {error}"),
        )
    })?
}

#[tauri::command]
async fn get_photograph_capture(
    app: AppHandle,
    shoot_name: String,
    relative_path: String,
) -> Result<image_processing::CaptureDetails, CatalogError> {
    let config = require_config_store(&app)?
        .get()
        .map_err(config_error_to_catalog)?;
    let catalog = config.catalog.ok_or_else(|| {
        CatalogError::new(
            "catalog_setup_required",
            "Create a Catalog before continuing.",
        )
    })?;

    tauri::async_runtime::spawn_blocking(move || {
        catalog::get_photograph_capture(&catalog.path, &catalog.name, &shoot_name, &relative_path)
    })
    .await
    .map_err(|error| {
        CatalogError::new(
            "photograph_capture_join",
            format!("Capture details could not be read: {error}"),
        )
    })?
}

#[tauri::command]
async fn update_shoot(
    app: AppHandle,
    shoot_name: String,
    new_name: String,
    medium: Option<ShootMedium>,
) -> Result<ShootDetails, CatalogError> {
    let catalog_path = require_ready_catalog_path(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        update_catalog_shoot(&catalog_path, &shoot_name, &new_name, medium)
    })
    .await
    .map_err(|error| {
        CatalogError::new(
            "shoot_update_join",
            format!("Shoot changes could not be applied: {error}"),
        )
    })?
}

#[tauri::command]
async fn set_jpeg_companion_link(
    app: AppHandle,
    shoot_name: String,
    raw_path: String,
    linked: bool,
) -> Result<ShootDetails, CatalogError> {
    let catalog_path = require_ready_catalog_path(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        catalog::set_jpeg_companion_link(&catalog_path, &shoot_name, &raw_path, linked)
    })
    .await
    .map_err(|error| {
        CatalogError::new(
            "pair_update_join",
            format!("Pair could not be updated: {error}"),
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
async fn inspect_source_medium(source_path: PathBuf) -> Result<SourceMediumHint, CatalogError> {
    tauri::async_runtime::spawn_blocking(move || catalog::inspect_source_medium(&source_path))
        .await
        .map_err(|error| {
            CatalogError::new(
                "source_scan",
                format!("Source check could not finish: {error}"),
            )
        })?
}

#[tauri::command]
async fn import_shoot(
    app: AppHandle,
    import_id: String,
    source_path: PathBuf,
    shoot_name: String,
    medium: Option<ShootMedium>,
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
            medium,
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

fn seed_development_catalog(app: &AppHandle, store: &AppConfigStore) {
    if !cfg!(debug_assertions) {
        return;
    }

    let Ok(config) = store.get() else {
        return;
    };
    if config.catalog.is_some() {
        return;
    }

    let legacy = app_config::legacy_worktree_config_path();
    let pictures = app.path().picture_dir().ok();
    let Some(pointer) = catalog::resolve_dev_catalog_seed(legacy.as_deref(), pictures.as_deref())
    else {
        return;
    };
    let _ = store.set_catalog(Some(pointer));
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
                        seed_development_catalog(app.handle(), &store);
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
            get_photograph_detail,
            get_photograph_capture,
            update_shoot,
            set_jpeg_companion_link,
            inspect_source_medium,
            purge_shoot,
            import_shoot,
            cancel_shoot_import,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
