// File: src/domain/models/region_4.rs

use crate::constants::*;
use crate::domain::errors::If97Error;
use crate::domain::state::{Region, WaterState};
use crate::regions::region_1::Region1;
use crate::regions::region_2::Region2;
use crate::regions::region_3::Region3;
use crate::regions::traits::WaterRegionModel;
use tracing::{debug, error, instrument, trace};

/// Коэффициенты для расчета линии насыщения[cite: 741].
const N: [f64; 10] = [
    0.11670521452767e4,
    -0.72421316703206e6,
    -0.17073846940092e2,
    0.12020824702470e5,
    -0.32325550322333e7,
    0.14915108613530e2,
    -0.48232657361591e4,
    0.40511340542057e6,
    -0.23855557567849,
    0.65017534844798e3,
];

/// Расчет давления насыщения $p_{sat}$ по заданной температуре $T$[cite: 742].
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

/// Расчет температуры насыщения $T_{sat}$ по заданному давлению $p$[cite: 747].
#[instrument(level = "trace")]
pub fn saturation_temperature(p: f64) -> f64 {
    if p < P_MIN_IF97 || p > P_C + 1e-3 {
        error!(
            p,
            "Давление вне диапазона линии насыщения (0.000611 - 22.064 МПа)"
        );
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

/// Расчет свойств двухфазной смеси (Регион 4).
///
/// Логика использует правило аддитивности $X = X_{liq} + x \cdot (X_{vap} - X_{liq})$,
/// определяя свойства на линиях жидкости и пара через смежные Регионы (1 и 2) или Регион 3 (около критики)[cite: 755].
#[instrument(level = "debug")]
pub fn calculate_two_phase(p: f64, x: f64) -> Result<WaterState, If97Error> {
    if !(0.0..=1.0).contains(&x) {
        error!(x, "Степень сухости x вне диапазона 0.0 - 1.0");
        return Err(If97Error::InvalidInput(
            "Степень сухости x должна быть в диапазоне от 0.0 до 1.0".into(),
        ));
    }
    let t_sat = saturation_temperature(p);
    if t_sat.is_nan() {
        return Err(If97Error::OutOfBounds(
            "Давление вне диапазона линии насыщения (0.000611 - 22.064 МПа)".into(),
        ));
    }

    let (state_liquid, state_vapor) = if t_sat <= 623.15 {
        trace!("Расчет граничных состояний (жидкость/пар) через Region1 и Region2");
        (
            Region1.calculate_pt(p, t_sat)?,
            Region2.calculate_pt(p, t_sat)?,
        )
    } else {
        let dt = T_C - t_sat;
        let fraction = (dt / 23.946).max(0.0).powf(0.65);
        let rho_guess_liq = RHO_C + 252.7 * fraction;
        let rho_guess_vap = RHO_C - 248.8 * fraction;
        trace!(
            rho_guess_liq,
            rho_guess_vap, "Расчет граничных состояний через Region3 (околокритическая зона)"
        );
        (
            Region3.calculate_pt_with_guess(p, t_sat, rho_guess_liq)?,
            Region3.calculate_pt_with_guess(p, t_sat, rho_guess_vap)?,
        )
    };

    let v_liq = state_liquid.v.inner();
    let v_vap = state_vapor.v.inner();

    let v = v_liq + x * (v_vap - v_liq);
    let h = state_liquid.h.inner() + x * (state_vapor.h.inner() - state_liquid.h.inner());
    let s = state_liquid.s.inner() + x * (state_vapor.s.inner() - state_liquid.s.inner());
    let u = h - (p * v * 1000.0);

    // Расчет скорости звука в смеси по формуле Вуда (замороженная модель)
    let w_liq = state_liquid.w.inner();
    let w_vap = state_vapor.w.inner();
    let w_mix = if x == 0.0 {
        w_liq
    } else if x == 1.0 {
        w_vap
    } else if w_liq.is_finite() && w_vap.is_finite() {
        // Формула Вуда через массовые доли и удельные объемы
        v / (x * (v_vap / w_vap).powi(2) + (1.0 - x) * (v_liq / w_liq).powi(2)).sqrt()
    } else {
        f64::NAN
    };

    // Теплоемкость при кипении стремится к бесконечности
    let cp_mix = f64::INFINITY;

    debug!(
        v,
        h, s, t_sat, u, w_mix, cp_mix, "Успешный расчет двухфазной области Region4"
    );
    Ok(WaterState {
        p: p.into(),
        t: t_sat.into(),
        v: v.into(),
        rho: (1.0 / v).into(),
        h: h.into(),
        s: s.into(),
        cp: cp_mix.into(),
        w: w_mix.into(),
        u: u.into(),
        region: Region::Region4,
    })
}
