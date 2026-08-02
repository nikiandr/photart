mod app_config;
mod photo_analysis;

use std::path::PathBuf;

use app_config::{AppConfig, AppConfigError, AppConfigStore};
use photo_analysis::{PhotoAnalysis, PhotoAnalysisError};
use tauri::{Manager, State};

#[tauri::command]
fn analyze_photo(path: String) -> Result<PhotoAnalysis, PhotoAnalysisError> {
    photo_analysis::analyze(&path)
}

#[tauri::command]
fn get_app_config(config: State<'_, AppConfigStore>) -> Result<AppConfig, AppConfigError> {
    config.get()
}

#[tauri::command]
fn set_catalogue_path(
    catalogue_path: Option<PathBuf>,
    config: State<'_, AppConfigStore>,
) -> Result<AppConfig, AppConfigError> {
    config.set_catalogue_path(catalogue_path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let config = AppConfigStore::open(app_config::config_path(app.handle())?)?;
            app.manage(config);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            analyze_photo,
            get_app_config,
            set_catalogue_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
