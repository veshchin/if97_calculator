mod app;
pub mod ui;
pub mod types;
pub mod plot; // <- Теперь компилятор видит содержимое src/plot.rs

fn main() {
    yew::Renderer::<app::App>::new().render();
}