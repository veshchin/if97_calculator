// Скрывает консоль на Windows при сборке релизной версии (не отладочной)
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! Entrypoint для desktop-обертки Tauri.

fn main() {
    // Вызов функции run() из lib.rs.
    // Замените `app_lib` на имя вашей библиотеки из src-tauri/Cargo.toml (обычно совпадает с именем пакета)
    app_lib::run();
}
