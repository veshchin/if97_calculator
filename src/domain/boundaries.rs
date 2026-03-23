use crate::state::Region;
use crate::region_4::saturation_pressure;

// Точная граница B23 по стандарту IAPWS-IF97
pub fn b23_pressure(t: f64) -> f64 {
    let n1 = 0.34805185628969e3;
    let n2 = -0.11671859879975e1;
    let n3 = 0.10192970039326e-2;
    n1 + n2 * t + n3 * t * t
}

pub fn determine_region(p: f64, t: f64) -> Region {
    if t < 273.15 || t > 2273.15 || p < 0.0 || p > 100.0 {
        return Region::OutOfBounds;
    }

    if t <= 623.15 {
        let p_sat = saturation_pressure(t);
        if p > p_sat { Region::Region1 } else { Region::Region2 }
    } else if t <= 863.15 {
        let p_b23 = b23_pressure(t);
        if p > p_b23 { Region::Region3 } else { Region::Region2 }
    } else if t <= 1073.15 {
        Region::Region2
    } else if p <= 50.0 {
        Region::Region5
    } else {
        Region::OutOfBounds
    }
}

// Дополнения в src/boundaries.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Region2Subregion {
    Region2a,
    Region2b,
    Region2c,
}

/// Граница B2ab на p-h диаграмме
pub fn boundary_2ab_enthalpy(p: f64) -> f64 {
    let n = [
        0.28313098393521e4, -0.21275304383186e2, -0.13813737351475e1,
        -0.53177894207904e-1, -0.11327150116631e-2, 0.14187640209428e-4
    ];
    let beta = p.sqrt();
    n[0] + n[1]*beta + n[2]*p + n[3]*beta.powi(3) + n[4]*p.powi(2) + n[5]*beta.powi(5)
}

/// Граница B2bc на p-h диаграмме
/// Уравнение границы B2bc (между подобластями 2b и 2c)
/// Вход: давление p [MPa]. Выход: энтальпия h [kJ/kg]
pub fn boundary_2bc_enthalpy(p: f64) -> f64 {
    // Нормализующие константы по стандарту IAPWS-IF97
    let p_star = 1.0; // MPa
    let h_star = 1.0; // kJ/kg

    // Точные коэффициенты из Уравнения 21 (Таблица 5 стандарта)
    let n3 = 0.12809002730136e-3;
    let n4 = 0.26526571908428e4;
    let n5 = 0.45257578905948e1;

    let pi = p / p_star;

    // Явное уравнение границы (B2bc)
    let eta = n4 + ((pi - n5) / n3).sqrt();

    eta * h_star
}

/// Граница B2ab на p-s диаграмме
pub fn boundary_2ab_entropy(p: f64) -> f64 {
    let n = [
        0.90584278514723e1, -0.67955786399241, 0.12809002730136e-1,
        -0.67292681825831e-3, 0.27102141570420e-4, -0.45381834360980e-6
    ];
    n[0] + n[1]*p.powf(0.4) + n[2]*p + n[3]*p.powf(2.4) + n[4]*p.powf(2.8) + n[5]*p.powf(3.2)
}

pub fn determine_region2_subregion_ph(p: f64, h: f64) -> Region2Subregion {
    if p <= 4.0 {
        Region2Subregion::Region2a
    } else if h >= boundary_2bc_enthalpy(p) { // Для границы 2b и 2c используется уравнение B2bc
        Region2Subregion::Region2b
    } else {
        Region2Subregion::Region2c
    }
}

pub fn determine_region2_subregion_ps(p: f64, s: f64) -> Region2Subregion {
    if p <= 4.0 {
        Region2Subregion::Region2a
    } else if s >= 5.85 { // Граница 2b и 2c для T(p,s) строго s = 5.85
        Region2Subregion::Region2b
    } else {
        Region2Subregion::Region2c
    }
}