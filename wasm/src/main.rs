/* File: src/main.rs */
mod app;
mod logger;
mod plot;
mod tauri_api;
mod types;
mod ui; // Подключаем наш модуль

use app::App;

fn main() {
    console_error_panic_hook::set_once();
    logger::init_logger();
    yew::Renderer::<App>::new().render();
}
