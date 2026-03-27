// File: src/domain/constants.rs

// --- Базовые физические константы IAPWS-IF97 ---
pub const R: f64 = 0.461526;         // Удельная газовая постоянная, kJ/(kg K)
pub const T_C: f64 = 647.096;        // Критическая температура, K
pub const P_C: f64 = 22.064;         // Критическое давление, MPa
pub const RHO_C: f64 = 322.0;        // Критическая плотность, kg/m^3

// --- Границы применимости стандарта ---
pub const T_MIN_IF97: f64 = 273.15;
pub const T_MAX_REGION1_2: f64 = 1073.15;
pub const T_MAX_REGION5: f64 = 2273.15;
pub const P_MIN_IF97: f64 = 0.000611212; // Тройная точка воды
pub const P_MAX_IF97: f64 = 100.0;
pub const P_MAX_REGION5: f64 = 50.0;

// --- Специфичные константы регионов (Параметры приведения) ---
pub const REGION1_P_STAR: f64 = 16.53;
pub const REGION1_T_STAR: f64 = 1386.0;

pub const REGION2_P_STAR: f64 = 1.0;
pub const REGION2_T_STAR: f64 = 540.0;

pub const REGION5_P_STAR: f64 = 1.0;
pub const REGION5_T_STAR: f64 = 1000.0;

// --- Настройки итерационных решателей ---
pub const SOLVER_TOLERANCE: f64 = 1e-7;
pub const SOLVER_TOLERANCE_TIGHT: f64 = 1e-8;
pub const SOLVER_DET_TOLERANCE: f64 = 1e-12;
pub const SOLVER_MAX_ITER_1D: i32 = 20;
pub const SOLVER_MAX_ITER_2D: i32 = 50;
pub const SOLVER_MAX_ITER_DENSITY: i32 = 100;