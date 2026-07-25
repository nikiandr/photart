mod photo_analysis;

use photo_analysis::{PhotoAnalysis, PhotoAnalysisError};

#[tauri::command]
fn analyze_photo(path: String) -> Result<PhotoAnalysis, PhotoAnalysisError> {
    photo_analysis::analyze(&path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![analyze_photo])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
