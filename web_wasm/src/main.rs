mod types;
mod plot;
mod ui;
mod app;

fn main() {
    yew::Renderer::<app::App>::new().render();
}