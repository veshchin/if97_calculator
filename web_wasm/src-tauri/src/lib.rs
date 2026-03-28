/* File: src-tauri/src/lib.rs */
use tauri_plugin_dialog::DialogExt;
use std::fs;
use base64::{Engine as _, engine::general_purpose::STANDARD};

#[tauri::command]
async fn load_file_dialog(app: tauri::AppHandle) -> Result<String, String> {
    // Добавлен фильтр для удобного выбора текстовых/табличных данных
    let file_path = app.dialog().file()
        .add_filter("Табличные данные", &["txt", "csv", "tsv", "dat"])
        .add_filter("Все файлы", &["*"])
        .blocking_pick_file();

    match file_path {
        Some(path) => {
            let p = path.into_path().map_err(|_| "Некорректный путь файла".to_string())?;
            fs::read_to_string(p).map_err(|e| e.to_string())
        },
        None => Err("Отменено пользователем".into())
    }
}

#[tauri::command]
async fn save_file_dialog(
    app: tauri::AppHandle,
    content: String,
    default_name: Option<String>,
    filter_name: Option<String>,
    filter_ext: Option<String>
) -> Result<(), String> {
    let mut dialog = app.dialog().file();

    // ОС автоматически подставит предложенное имя
    if let Some(name) = default_name {
        dialog = dialog.set_file_name(name);
    }

    // ОС автоматически добавит нужное расширение при сохранении
    if let (Some(fname), Some(fext)) = (filter_name, filter_ext) {
        // ИСПРАВЛЕНИЕ ЗДЕСЬ: Преобразуем String в &str через .as_str()
        dialog = dialog.add_filter(fname, &[fext.as_str()]);
    }

    let file_path = dialog.blocking_save_file();

    match file_path {
        Some(path) => {
            let p = path.into_path().map_err(|_| "Некорректный путь файла".to_string())?;
            fs::write(p, content).map_err(|e| e.to_string())
        },
        None => Err("Отменено пользователем".into())
    }
}

#[tauri::command]
async fn save_plot_dialog(app: tauri::AppHandle, b64: String) -> Result<(), String> {
    let file_path = app.dialog().file()
        .set_file_name("if97_plot.png")
        .add_filter("Изображение PNG", &["png"])
        .blocking_save_file();

    match file_path {
        Some(path) => {
            let p = path.into_path().map_err(|_| "Некорректный путь файла".to_string())?;
            let image_bytes = STANDARD.decode(&b64).map_err(|e| e.to_string())?;
            fs::write(p, image_bytes).map_err(|e| e.to_string())
        },
        None => Err("Отменено пользователем".into())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            load_file_dialog,
            save_file_dialog,
            save_plot_dialog
        ])
        .run(tauri::generate_context!())
        .expect("Ошибка при запуске приложения Tauri");
}