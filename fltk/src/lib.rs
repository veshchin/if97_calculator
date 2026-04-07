// File: src/lib.rs

pub mod plot;
pub mod state;
pub mod ui;

// Переэкспортируем важные типы для удобства доступа
pub use state::{AppState, Message, SavedData};
