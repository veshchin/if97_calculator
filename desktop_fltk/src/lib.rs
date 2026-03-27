// File: src/lib.rs

pub mod state;
pub mod ui;
pub mod plot;

// Переэкспортируем важные типы для удобства доступа
pub use state::{AppState, Message, PlotType, SavedData};