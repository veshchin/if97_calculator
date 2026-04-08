//! Web-frontend `if97_calculator` на базе Yew.
//!
//! В режиме Tauri этот frontend взаимодействует с backend через команды `tauri`.
mod app;
mod logger;
mod plot;
mod tauri_api;
mod types;
mod ui;

use app::App;

fn main() {
    console_error_panic_hook::set_once();
    logger::init_logger();
    yew::Renderer::<App>::new().render();
}
