// core/src/domain/boundaries.rs

//! Модуль определения границ регионов по стандарту IAPWS-IF97.
//!
//! Содержит логику маршрутизации (в какой регион попадает заданная точка p-T),
//! а также уравнения для внутренних границ, таких как граница B23.

use crate::domain::state::Region;
use crate::topology::phase_equilibrium::saturation_pressure;
use tracing::{instrument, trace};

/// Точная граница B23 по стандарту IAPWS-IF97 (между Регионом 2 и Регионом 3).
/// Возвращает давление в МПа по заданной температуре в К.
#[instrument(level = "trace")]
pub fn b23_pressure(t: f64) -> f64 {
    let n1 = 0.34805185628969e3;
    let n2 = -0.11671859879975e1;
    let n3 = 0.10192970039326e-2;
    n1 + n2 * t + n3 * t * t
}

/// Обратная форма границы B23: возвращает температуру в К по давлению в МПа.
#[instrument(level = "trace")]
pub fn b23_temperature(p: f64) -> f64 {
    let n3 = 0.10192970039326e-2;
    let n4 = 0.57254459862746e3;
    let n5 = 0.13918839778870e2;
    n4 + ((p - n5) / n3).sqrt()
}

/// Основная функция маршрутизации: определяет регион по давлению и температуре.
#[instrument(level = "trace")]
pub fn determine_region(p: f64, t: f64) -> Region {
    if t < 273.15 || t > 2273.15 || p < 0.0 || p > 100.0 {
        trace!("Точка вне границ стандарта IAPWS-IF97");
        return Region::OutOfBounds;
    }

    // Линия насыщения (Регион 4) существует вплоть до критической температуры (647.096 К).
    // Проверяем эту границу до разделения на остальные регионы.
    if t <= 647.096 {
        let p_sat = saturation_pressure(t);
        // Используем допуск 1e-5 для надежного захвата точек около критической зоны
        if ((p - p_sat) / p_sat).abs() < 1e-5 {
            trace!("Точка лежит на линии насыщения (Region4)");
            return Region::Region4;
        }
    }

    if t <= 623.15 {
        let p_sat = saturation_pressure(t);
        if p > p_sat {
            trace!("Определен Region1");
            return Region::Region1;
        } else {
            trace!("Определен Region2");
            return Region::Region2;
        }
    } else if t <= 863.15 {
        let p_b23 = b23_pressure(t);
        if p >= p_b23 * (1.0 - 1e-10) {
            trace!("Определен Region3");
            Region::Region3
        } else {
            trace!("Определен Region2");
            Region::Region2
        }
    } else if t <= 1073.15 {
        trace!("Определен Region2");
        Region::Region2
    } else if p <= 50.0 {
        trace!("Определен Region5");
        Region::Region5
    } else {
        trace!("Точка вне границ стандарта IAPWS-IF97");
        Region::OutOfBounds
    }
}

/// Внутренние субрегионы Региона 2.
/// Используются для выбора правильных наборов коэффициентов в обратных уравнениях.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Region2Subregion {
    Region2a,
    Region2b,
    Region2c,
}

/// Граница энтальпии (в кДж/кг) между субрегионами 2b и 2c в зависимости от давления.
#[instrument(level = "trace")]
pub fn boundary_2bc_enthalpy(p: f64) -> f64 {
    let n3 = 0.12809002730136e-3;
    let n4 = 0.26526571908428e4;
    let n5 = 0.45257578905948e1;

    if p < n5 {
        return n4;
    }

    n4 + ((p - n5) / n3).sqrt()
}

/// Определение субрегиона Региона 2 по давлению и энтальпии.
#[instrument(level = "trace")]
pub fn determine_region2_subregion_ph(p: f64, h: f64) -> Region2Subregion {
    if p <= 4.0 {
        trace!("Определен субрегион Region2a");
        Region2Subregion::Region2a
    } else if h >= boundary_2bc_enthalpy(p) {
        trace!("Определен субрегион Region2b");
        Region2Subregion::Region2b
    } else {
        trace!("Определен субрегион Region2c");
        Region2Subregion::Region2c
    }
}

/// Определение субрегиона Региона 2 по давлению и энтропии.
#[instrument(level = "trace")]
pub fn determine_region2_subregion_ps(p: f64, s: f64) -> Region2Subregion {
    if p <= 4.0 {
        trace!("Определен субрегион Region2a");
        Region2Subregion::Region2a
    } else if s >= 5.85 {
        trace!("Определен субрегион Region2b");
        Region2Subregion::Region2b
    } else {
        trace!("Определен субрегион Region2c");
        Region2Subregion::Region2c
    }
}
