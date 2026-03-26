// core/src/domain/boundaries.rs

use crate::domain::state::Region;
use crate::domain::models::region_4::saturation_pressure;

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

    // Линия насыщения (Регион 4) существует вплоть до критической температуры (647.096 К).
    // Проверяем эту границу до разделения на остальные регионы.
    if t <= 647.096 {
        let p_sat = saturation_pressure(t);
        // Используем допуск 1e-5 для надежного захвата точек около критической зоны
        if ((p - p_sat) / p_sat).abs() < 1e-5 {
            return Region::Region4;
        }
    }

    if t <= 623.15 {
        let p_sat = saturation_pressure(t);
        if p > p_sat {
            return Region::Region1;
        } else {
            return Region::Region2;
        }
    } else if t <= 863.15 {
        let p_b23 = b23_pressure(t);
        if p >= p_b23 * (1.0 - 1e-10) {
            Region::Region3
        } else {
            Region::Region2
        }
    } else if t <= 1073.15 {
        Region::Region2
    } else if p <= 50.0 {
        Region::Region5
    } else {
        Region::OutOfBounds
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Region2Subregion {
    Region2a,
    Region2b,
    Region2c,
}

pub fn boundary_2bc_enthalpy(p: f64) -> f64 {
    let n3 = 0.12809002730136e-3;
    let n4 = 0.26526571908428e4;
    let n5 = 0.45257578905948e1;

    if p < n5 {
        return n4;
    }

    n4 + ((p - n5) / n3).sqrt()
}

pub fn determine_region2_subregion_ph(p: f64, h: f64) -> Region2Subregion {
    if p <= 4.0 {
        Region2Subregion::Region2a
    } else if h >= boundary_2bc_enthalpy(p) {
        Region2Subregion::Region2b
    } else {
        Region2Subregion::Region2c
    }
}

pub fn determine_region2_subregion_ps(p: f64, s: f64) -> Region2Subregion {
    if p <= 4.0 {
        Region2Subregion::Region2a
    } else if s >= 5.85 {
        Region2Subregion::Region2b
    } else {
        Region2Subregion::Region2c
    }
}