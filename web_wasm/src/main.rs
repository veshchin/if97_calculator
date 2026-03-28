/* File: src/main.rs */
mod app;
mod ui;
mod types;
mod plot;
mod logger; // Подключаем наш модуль

use app::App;

fn main() {
    // Инициализируем перехват логов tracing ДО старта Yew
    logger::init_logger();

    // Можно сразу бросить тестовый лог
    tracing::info!("Tracing logger initialized successfully.");

    yew::Renderer::<App>::new().render();
}