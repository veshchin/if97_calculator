// src/backward.rs

use crate::tables::{BACKWARD1_T_PH, BACKWARD1_T_PS, BACKWARD1_P_HS};

/// Расчет температуры T(p, h) для Региона 1
/// Вход: p [MPa], h [kJ/kg]. Выход: T [K]
pub fn region1_t_ph(p: f64, h: f64) -> f64 {
    // Нормализующие константы по стандарту IAPWS-IF97
    let p_star = 1.0;      // MPa
    let h_star = 2500.0;   // kJ/kg
    let t_star = 1.0;      // K

    let pi = p / p_star;
    let eta = h / h_star;

    // Для T(p,h) в Регионе 1 IAPWS предписывает использовать (eta + 1)
    let eta_term = eta + 1.0;

    let mut t_ratio = 0.0;
    for &(n, i, j) in BACKWARD1_T_PH.iter() {
        t_ratio += n * pi.powi(i) * eta_term.powi(j);
    }

    t_ratio * t_star
}

/// Расчет температуры T(p, s) для Региона 1
/// Вход: p [MPa], s [kJ/(kg K)]. Выход: T [K]
pub fn region1_t_ps(p: f64, s: f64) -> f64 {
    // Нормализующие константы по стандарту IAPWS-IF97
    let p_star = 1.0;      // MPa
    let s_star = 1.0;      // kJ/(kg K)
    let t_star = 1.0;      // K

    let pi = p / p_star;
    let sigma = s / s_star;

    // Для T(p,s) в Регионе 1 IAPWS предписывает использовать (sigma + 2)
    let sigma_term = sigma + 2.0;

    let mut t_ratio = 0.0;
    for &(n, i, j) in BACKWARD1_T_PS.iter() {
        t_ratio += n * pi.powi(i) * sigma_term.powi(j);
    }

    t_ratio * t_star
}

/// Расчет давления p(h, s) для Региона 1
/// Вход: h [kJ/kg], s [kJ/(kg K)]. Выход: p [MPa]
pub fn region1_p_hs(h: f64, s: f64) -> f64 {
    // Нормализующие константы по стандарту IAPWS-IF97 (обратите внимание, они другие!)
    let h_star = 3400.0;   // kJ/kg
    let s_star = 7.6;      // kJ/(kg K)
    let p_star = 100.0;    // MPa

    let eta = h / h_star;
    let sigma = s / s_star;

    // В уравнении p(h,s) для Региона 1 используются eta и sigma без смещений
    let mut p_ratio = 0.0;
    for &(n, i, j) in BACKWARD1_P_HS.iter() {
        p_ratio += n * eta.powi(i) * sigma.powi(j);
    }

    p_ratio * p_star
}

// Дополнения в src/backward.rs

use crate::tables::{
    BACKWARD2A_T_PH, BACKWARD2B_T_PH, BACKWARD2C_T_PH,
    BACKWARD2A_T_PS, BACKWARD2B_T_PS, BACKWARD2C_T_PS
};
use crate::boundaries::{Region2Subregion, determine_region2_subregion_ph, determine_region2_subregion_ps};

pub fn region2_t_ph(p: f64, h: f64) -> f64 {
    match determine_region2_subregion_ph(p, h) {
        Region2Subregion::Region2a => region2a_t_ph(p, h),
        Region2Subregion::Region2b => region2b_t_ph(p, h),
        Region2Subregion::Region2c => region2c_t_ph(p, h),
    }
}

pub fn region2_t_ps(p: f64, s: f64) -> f64 {
    match determine_region2_subregion_ps(p, s) {
        Region2Subregion::Region2a => region2a_t_ps(p, s),
        Region2Subregion::Region2b => region2b_t_ps(p, s),
        Region2Subregion::Region2c => region2c_t_ps(p, s),
    }
}

// --- Функции для T(p, h) ---

fn region2a_t_ph(p: f64, h: f64) -> f64 {
    let pi = p / 1.0;
    let eta = h / 2000.0;
    let mut t_ratio = 0.0;
    for &(n, i, j) in BACKWARD2A_T_PH.iter() {
        t_ratio += n * pi.powi(i) * (eta - 2.1).powi(j);
    }
    t_ratio * 1.0 // T* = 1 K
}

// В src/backward.rs

// --- Исправления T(p, h) ---

fn region2b_t_ph(p: f64, h: f64) -> f64 {
    let pi = p / 1.0;
    let eta = h / 2000.0;
    let mut t_ratio = 0.0;
    for &(n, i, j) in BACKWARD2B_T_PH.iter() {
        // Уравнение (23): смещение (pi - 2.0) и (eta - 2.6)
        t_ratio += n * (pi - 2.0).powi(i) * (eta - 2.6).powi(j);
    }
    t_ratio * 1.0
}

fn region2c_t_ph(p: f64, h: f64) -> f64 {
    let pi = p / 1.0;
    let eta = h / 2000.0;
    let mut t_ratio = 0.0;
    for &(n, i, j) in BACKWARD2C_T_PH.iter() {
        // Уравнение (24): смещение (pi + 25.0) и (eta - 1.8)
        t_ratio += n * (pi + 25.0).powi(i) * (eta - 1.8).powi(j);
    }
    t_ratio * 1.0
}


// --- Исправления T(p, s) ---

fn region2b_t_ps(p: f64, s: f64) -> f64 {
    let pi = p / 1.0;
    let sigma = s / 0.7853;
    let mut t_ratio = 0.0;
    for &(n, i, j) in BACKWARD2B_T_PS.iter() {
        // Уравнение 26: смещение должно быть (10.0 - sigma)
        t_ratio += n * pi.powi(i) * (10.0 - sigma).powi(j);
    }
    t_ratio * 1.0
}

fn region2c_t_ps(p: f64, s: f64) -> f64 {
    let pi = p / 1.0;
    let sigma = s / 2.9251;
    let mut t_ratio = 0.0;
    for &(n, i, j) in BACKWARD2C_T_PS.iter() {
        // Уравнение 27: смещение должно быть (2.0 - sigma)
        t_ratio += n * pi.powi(i) * (2.0 - sigma).powi(j);
    }
    t_ratio * 1.0
}

// --- Функции для T(p, s) ---

fn region2a_t_ps(p: f64, s: f64) -> f64 {
    let pi = p / 1.0;
    let sigma = s / 2.0;
    let mut t_ratio = 0.0;
    for &(n, i, j) in BACKWARD2A_T_PS.iter() {
        // ВАЖНО: i должно быть f64, так как степени I_i здесь дробные!
        t_ratio += n * pi.powf(i) * (sigma - 2.0).powi(j);
    }
    t_ratio * 1.0
}

use crate::region_5::Region5;
use crate::math::ThermodynamicRegion;
use crate::constants::R;

/// Обратный расчет температуры T(p, h) для Региона 5 (1D метод Ньютона)
/// Вход: p [MPa], h_target [kJ/kg].
/// Выход: T [K] или ошибка невыполнения сходимости.
pub fn region5_t_ph(p: f64, h_target: f64) -> Result<f64, &'static str> {
    let region5 = Region5;
    let p_star = 1.0;
    let t_star = 1000.0;
    let pi = p / p_star;

    // Стартуем с медианной температуры для Региона 5
    let mut t = 1500.0;
    let tol = 1e-7;
    let max_iters = 20;

    for _ in 0..max_iters {
        let tau = t_star / t;

        let gamma_tau = region5.gamma_tau(pi, tau);
        let gamma_tau_tau = region5.gamma_tau_tau(pi, tau);

        // Текущая энтальпия и теплоемкость (производная энтальпии по T)
        let h_curr = R * t * tau * gamma_tau;
        let cp_curr = -R * tau.powi(2) * gamma_tau_tau;

        let f = h_curr - h_target;

        if f.abs() < tol {
            return Ok(t);
        }

        // Шаг метода Ньютона
        let dt = f / cp_curr;
        let next_t = t - dt;

        // Clamping: Жестко ограничиваем температуру физическими рамками Региона 5
        t = next_t.clamp(1073.15, 2273.15);
    }

    Err("Превышен лимит итераций в решателе Region 5 (p, h).")
}

/// Обратный расчет температуры T(p, s) для Региона 5 (1D метод Ньютона)
/// Вход: p [MPa], s_target [kJ/(kg K)].
/// Выход: T [K] или ошибка невыполнения сходимости.
pub fn region5_t_ps(p: f64, s_target: f64) -> Result<f64, &'static str> {
    let region5 = Region5;
    let p_star = 1.0;
    let t_star = 1000.0;
    let pi = p / p_star;

    let mut t = 1500.0;
    let tol = 1e-7;
    let max_iters = 20;

    for _ in 0..max_iters {
        let tau = t_star / t;

        let gamma = region5.gamma(pi, tau);
        let gamma_tau = region5.gamma_tau(pi, tau);
        let gamma_tau_tau = region5.gamma_tau_tau(pi, tau);

        // Текущая энтропия
        let s_curr = R * (tau * gamma_tau - gamma);

        // Теплоемкость и производная энтропии по температуре (ds/dT = cp/T)
        let cp_curr = -R * tau.powi(2) * gamma_tau_tau;
        let ds_dt = cp_curr / t;

        let f = s_curr - s_target;

        if f.abs() < tol {
            return Ok(t);
        }

        let dt = f / ds_dt;
        let next_t = t - dt;

        t = next_t.clamp(1073.15, 2273.15);
    }

    Err("Превышен лимит итераций в решателе Region 5 (p, s).")
}