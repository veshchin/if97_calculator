use if97_core::domain::calculator::Calculator;
use if97_core::domain::state::WaterState;
use tauri_plugin_shell; // Явное указание на использование плагина

#[tauri::command]
fn calculate_pt_command(p: f64, t: f64) -> Result<WaterState, String> {
    Calculator::calculate_pt(p, t).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![calculate_pt_command])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}