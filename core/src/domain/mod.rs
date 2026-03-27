// File: src/domain/mod.rs
pub mod state;
pub mod constants;
pub mod errors;      // Добавили наш новый модуль ошибок
pub mod traits;
pub mod math;       // Добавили модуль с математическими трейтами
pub mod boundaries;
pub mod models;
pub mod calculator; // Теперь это наш единый фасад If97
pub mod tables;