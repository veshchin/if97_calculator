// File: src/domain/models/region_4.rs

use crate::domain::state::{Region, WaterState};
use crate::domain::models::region_1::Region1;
use crate::domain::models::region_2::Region2;
use crate::domain::models::region_3::Region3; // Импортируем Region3
use crate::domain::traits::WaterRegionModel;
use tracing::{instrument, trace, debug, error};

const N: [f64; 10] = [
    0.11670521452767e4, -0.72421316703206e6, -0.17073846940092e2,
    0.12020824702470e5, -0.32325550322333e7, 0.14915108613530e2,
    -0.48232657361591e4, 0.40511340542057e6, -0.23855557567849,
    0.65017534844798e3
];

#[instrument(level = "trace")]
pub fn saturation_pressure(t: f64) -> f64 {
    let v = t + N[8] / (t - N[9]);
    let a = v.powi(2) + N[0] * v + N[1];
    let b = N[2] * v.powi(2) + N[3] * v + N[4];
    let c = N[5] * v.powi(2) + N[6] * v + N[7];
    let p_ratio = (2.0 * c) / (-b + (b.powi(2) - 4.0 * a * c).sqrt());
    let p = p_ratio.powi(4);
    trace!(p, "Вычислено давление насыщения (p_sat)");
    p
}

#[instrument(level = "trace")]
pub fn saturation_temperature(p: f64) -> f64 {
    if p < 0.000611212 || p > 22.064001 {
        error!(p, "Давление вне диапазона линии насыщения (0.000611 - 22.064 МПа)");
        return f64::NAN;
    }
    let beta = p.sqrt().sqrt();
    let beta2 = beta.powi(2);
    let e = beta2 + N[2] * beta + N[5];
    let f = N[0] * beta2 + N[3] * beta + N[6];
    let g = N[1] * beta2 + N[4] * beta + N[7];
    let d = (2.0 * g) / (-f - (f.powi(2) - 4.0 * e * g).sqrt());
    let n9 = N[8];
    let n10 = N[9];
    let t = (n10 + d - ((n10 + d).powi(2) - 4.0 * (n9 + n10 * d)).sqrt()) / 2.0;
    trace!(t, "Вычислена температура насыщения (t_sat)");
    t
}

#[instrument(level = "debug")]
pub fn calculate_two_phase(p: f64, x: f64) -> Result<WaterState, &'static str> {
    if !(0.0..=1.0).contains(&x) {
        error!(x, "Степень сухости x вне диапазона 0.0 - 1.0");
        return Err("Степень сухости x должна быть в диапазоне от 0.0 до 1.0");
    }
    let t_sat = saturation_temperature(p);
    if t_sat.is_nan() {
        return Err("Давление вне диапазона линии насыщения (0.000611 - 22.064 МПа)");
    }

    let (state_liquid, state_vapor) = if t_sat <= 623.15 {
        trace!("Расчет граничных состояний (жидкость/пар) через Region1 и Region2");
        (Region1.calculate_pt(p, t_sat)?, Region2.calculate_pt(p, t_sat)?)
    } else {
        let dt = 647.096 - t_sat;
        // Улучшенная аппроксимация (степенная функция 0.65) для нелинейной околокритической зоны
        let fraction = (dt / 23.946).max(0.0).powf(0.65);
        let rho_guess_liq = 322.0 + 252.7 * fraction;
        let rho_guess_vap = 322.0 - 248.8 * fraction;
        trace!(rho_guess_liq, rho_guess_vap, "Расчет граничных состояний через Region3 (околокритическая зона)");
        (
            Region3.calculate_pt_with_guess(p, t_sat, rho_guess_liq)?,
            Region3.calculate_pt_with_guess(p, t_sat, rho_guess_vap)?
        )
    };

    let v = state_liquid.v + x * (state_vapor.v - state_liquid.v);
    let h = state_liquid.h + x * (state_vapor.h - state_liquid.h);
    let s = state_liquid.s + x * (state_vapor.s - state_liquid.s);

    debug!(v, h, s, t_sat, "Успешный расчет двухфазной области Region4");
    Ok(WaterState {
        p, t: t_sat, v,
        rho: 1.0 / v, h, s,
        cp: f64::NAN, w: f64::NAN,
        region: Region::Region4
    })
}