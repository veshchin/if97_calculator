//! Утилиты для работы с линией насыщения (Region 4).
//!
//! Основной вычислительный API ядра (`If97::px`) возвращает свойства влажного пара и,
//! по определению, рассчитывает обе граничные фазы (жидкость и пар) для последующей
//! интерполяции по x.
//!
//! Для построения диаграмм часто нужны именно граничные кривые (x=0 и x=1). В этом
//! случае рассчитывать обе фазы на каждой точке избыточно. Этот модуль предоставляет
//! быстрые функции для получения насыщенной жидкости/пара по заданному давлению.

use crate::constants::{RHO_C, T_C};
use crate::domain::errors::If97Error;
use crate::domain::state::WaterState;
use crate::domain::units::{Kelvin, MegaPascal};
use crate::regions::region_1::Region1;
use crate::regions::region_2::Region2;
use crate::regions::region_3::Region3;
use crate::regions::traits::WaterRegionModel;
use crate::topology::phase_equilibrium::saturation_temperature;

fn sat_boundary_temperature(pressure: f64) -> Result<f64, If97Error> {
    let t_sat = saturation_temperature(pressure);
    if t_sat.is_nan() {
        return Err(If97Error::OutOfBounds(
            "Давление вне диапазона линии насыщения (0.000611 - 22.064 МПа)".into(),
        ));
    }
    Ok(t_sat)
}

/// Температура насыщения `T_sat(p)` на линии Region 4.
pub fn saturation_t(pressure: MegaPascal) -> Result<Kelvin, If97Error> {
    sat_boundary_temperature(pressure.inner()).map(Kelvin)
}

/// Насыщенная жидкость (x=0) на линии Region 4 по давлению.
pub fn saturated_liquid(pressure: MegaPascal) -> Result<WaterState, If97Error> {
    let p = pressure.inner();
    let t_sat = sat_boundary_temperature(p)?;

    if t_sat <= 623.15 {
        // Граница Region 1.
        Region1.calculate_pt(p, t_sat)
    } else {
        // Околокритическая зона: обе ветви лежат в Region 3.
        let dt = T_C - t_sat;
        let fraction = (dt / 23.946).max(0.0).powf(0.65);
        let rho_guess_liq = RHO_C + 252.7 * fraction;
        Region3.calculate_pt_with_guess(p, t_sat, rho_guess_liq)
    }
}

/// Насыщенный пар (x=1) на линии Region 4 по давлению.
pub fn saturated_vapor(pressure: MegaPascal) -> Result<WaterState, If97Error> {
    let p = pressure.inner();
    let t_sat = sat_boundary_temperature(p)?;

    if t_sat <= 623.15 {
        // Граница Region 2.
        Region2.calculate_pt(p, t_sat)
    } else {
        let dt = T_C - t_sat;
        let fraction = (dt / 23.946).max(0.0).powf(0.65);
        let rho_guess_vap = RHO_C - 248.8 * fraction;
        Region3.calculate_pt_with_guess(p, t_sat, rho_guess_vap)
    }
}

