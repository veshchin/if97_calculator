// File: src/domain/models/region_1.rs

use crate::constants::*;
use crate::domain::errors::If97Error;
use crate::domain::state::{Region, WaterState};
use crate::regions::math::GibbsRegion;
use crate::regions::traits::WaterRegionModel;
use crate::tables::{BACKWARD1_T_PH, BACKWARD1_T_PS, REGION1};
use tracing::{debug, error, instrument, trace};

/// Модель Региона 1 (жидкая вода) по стандарту IAPWS-IF97.
///
/// Использует фундаментальное уравнение состояния через энергию Гиббса.
pub struct Region1;

impl Region1 {
    #[inline(always)]
    #[instrument(level = "trace")]
    fn precompute_pi_powers(pi_term: f64) -> [f64; 35] {
        let mut powers = [0.0; 35];
        for i in 0..35 {
            powers[i] = pi_term.powi(i as i32);
        }
        powers
    }

    #[inline(always)]
    #[instrument(level = "trace")]
    fn precompute_tau_powers(tau_term: f64) -> [f64; 61] {
        let mut powers = [0.0; 61];
        for j in -43..=17 {
            powers[(j + 43) as usize] = tau_term.powi(j as i32);
        }
        powers
    }

    /// Обратное уравнение для расчета температуры $T(p, h)$ в Регионе 1.
    /// Позволяет избежать итерационных расчетов.
    #[instrument(level = "trace")]
    fn calc_t_ph(p: f64, h: f64) -> f64 {
        let pi = p / 1.0;
        let eta_term = (h / 2500.0) + 1.0;
        let mut t_ratio = 0.0;
        for &(n, i, j) in BACKWARD1_T_PH.iter() {
            t_ratio += n * pi.powi(i) * eta_term.powi(j);
        }
        let t = t_ratio * 1.0;
        trace!(
            t,
            "Вычислена температура по обратному уравнению (p, h) для Region1"
        );
        t
    }

    /// Обратное уравнение для расчета температуры $T(p, s)$ в Регионе 1.
    /// Позволяет избежать итерационных расчетов.
    #[instrument(level = "trace")]
    fn calc_t_ps(p: f64, s: f64) -> f64 {
        let pi = p / 1.0;
        let sigma_term = (s / 1.0) + 2.0;
        let mut t_ratio = 0.0;
        for &(n, i, j) in BACKWARD1_T_PS.iter() {
            t_ratio += n * pi.powi(i) * sigma_term.powi(j);
        }
        let t = t_ratio * 1.0;
        trace!(
            t,
            "Вычислена температура по обратному уравнению (p, s) для Region1"
        );
        t
    }
}

impl GibbsRegion for Region1 {
    #[instrument(level = "trace", skip(self))]
    fn gamma(&self, pi: f64, tau: f64) -> f64 {
        let pi_term = 7.1 - pi;
        let tau_term = tau - 1.222;
        let pi_powers = Self::precompute_pi_powers(pi_term);
        let tau_powers = Self::precompute_tau_powers(tau_term);
        let res = REGION1.iter().fold(0.0, |acc, &(n, i, j)| {
            acc + n * pi_powers[i as usize] * tau_powers[(j + 43) as usize]
        });
        trace!(res, "Рассчитана базовая gamma");
        res
    }

    #[instrument(level = "trace", skip(self))]
    fn gamma_pi(&self, pi: f64, tau: f64) -> f64 {
        let pi_term = 7.1 - pi;
        let tau_term = tau - 1.222;
        let pi_powers = Self::precompute_pi_powers(pi_term);
        let tau_powers = Self::precompute_tau_powers(tau_term);
        let res = REGION1.iter().fold(0.0, |acc, &(n, i, j)| {
            if i == 0 {
                return acc;
            }
            acc - n * (i as f64) * pi_powers[(i - 1) as usize] * tau_powers[(j + 43) as usize]
        });
        trace!(res, "Рассчитана производная gamma_pi");
        res
    }

    #[instrument(level = "trace", skip(self))]
    fn gamma_tau(&self, pi: f64, tau: f64) -> f64 {
        let pi_term = 7.1 - pi;
        let tau_term = tau - 1.222;
        let pi_powers = Self::precompute_pi_powers(pi_term);
        let tau_powers = Self::precompute_tau_powers(tau_term);
        let res = REGION1.iter().fold(0.0, |acc, &(n, i, j)| {
            if j == 0 {
                return acc;
            }
            acc + n * pi_powers[i as usize] * (j as f64) * tau_powers[(j - 1 + 43) as usize]
        });
        trace!(res, "Рассчитана производная gamma_tau");
        res
    }

    #[instrument(level = "trace", skip(self))]
    fn gamma_pi_pi(&self, pi: f64, tau: f64) -> f64 {
        let pi_term = 7.1 - pi;
        let tau_term = tau - 1.222;
        let pi_powers = Self::precompute_pi_powers(pi_term);
        let tau_powers = Self::precompute_tau_powers(tau_term);
        let res = REGION1.iter().fold(0.0, |acc, &(n, i, j)| {
            if i <= 1 {
                return acc;
            }
            acc + n
                * (i as f64)
                * ((i - 1) as f64)
                * pi_powers[(i - 2) as usize]
                * tau_powers[(j + 43) as usize]
        });
        trace!(res, "Рассчитана производная gamma_pi_pi");
        res
    }

    #[instrument(level = "trace", skip(self))]
    fn gamma_tau_tau(&self, pi: f64, tau: f64) -> f64 {
        let pi_term = 7.1 - pi;
        let tau_term = tau - 1.222;
        let pi_powers = Self::precompute_pi_powers(pi_term);
        let tau_powers = Self::precompute_tau_powers(tau_term);
        let res = REGION1.iter().fold(0.0, |acc, &(n, i, j)| {
            if j == 0 || j == 1 {
                return acc;
            }
            acc + n
                * pi_powers[i as usize]
                * (j as f64)
                * ((j - 1) as f64)
                * tau_powers[(j - 2 + 43) as usize]
        });
        trace!(res, "Рассчитана производная gamma_tau_tau");
        res
    }

    #[instrument(level = "trace", skip(self))]
    fn gamma_pi_tau(&self, pi: f64, tau: f64) -> f64 {
        let pi_term = 7.1 - pi;
        let tau_term = tau - 1.222;
        let pi_powers = Self::precompute_pi_powers(pi_term);
        let tau_powers = Self::precompute_tau_powers(tau_term);
        let res = REGION1.iter().fold(0.0, |acc, &(n, i, j)| {
            if i == 0 || j == 0 {
                return acc;
            }
            acc - n
                * (i as f64)
                * pi_powers[(i - 1) as usize]
                * (j as f64)
                * tau_powers[(j - 1 + 43) as usize]
        });
        trace!(res, "Рассчитана смешанная производная gamma_pi_tau");
        res
    }
}

impl WaterRegionModel for Region1 {
    #[instrument(level = "debug", skip(self))]
    fn calculate_pt(&self, p: f64, t: f64) -> Result<WaterState, If97Error> {
        let pi = p / REGION1_P_STAR;
        let tau = REGION1_T_STAR / t;
        trace!(pi, tau, "Приведенные параметры");

        let gamma = self.gamma(pi, tau);
        let gamma_pi = self.gamma_pi(pi, tau);
        let gamma_tau = self.gamma_tau(pi, tau);
        let gamma_pi_pi = self.gamma_pi_pi(pi, tau);
        let gamma_tau_tau = self.gamma_tau_tau(pi, tau);
        let gamma_pi_tau = self.gamma_pi_tau(pi, tau);

        let v = 0.001 * (R * t / p) * pi * gamma_pi;
        let h = R * t * tau * gamma_tau;
        let s = R * (tau * gamma_tau - gamma);
        let cp = -R * tau.powi(2) * gamma_tau_tau;

        let w_squared = (R * t * gamma_pi.powi(2))
            / ((gamma_pi - tau * gamma_pi_tau).powi(2) / (tau.powi(2) * gamma_tau_tau)
                - gamma_pi_pi);
        let w = if w_squared > 0.0 {
            (w_squared * 1000.0).sqrt()
        } else {
            f64::NAN
        };
        let u = h - (p * v * 1000.0);

        debug!(
            v,
            h, s, cp, w, u, "Успешный прямой расчет свойств Region1 (p, t)"
        );
        Ok(WaterState {
            p: p.into(),
            t: t.into(),
            v: v.into(),
            rho: (1.0 / v).into(),
            h: h.into(),
            s: s.into(),
            cp: cp.into(),
            w: w.into(),
            u: u.into(),
            region: Region::Region1,
        })
    }

    #[instrument(level = "debug", skip(self))]
    fn calculate_ph(&self, p: f64, h: f64) -> Result<WaterState, If97Error> {
        let t = Self::calc_t_ph(p, h);
        if t < T_MIN_IF97 || t > 623.15 || p > P_MAX_IF97 {
            error!(t, "Вычисленная температура {}K вне границ Региона 1", t);
            return Err(If97Error::OutOfBounds(
                "Точка (p, h) лежит вне границ Региона 1".into(),
            ));
        }
        debug!(
            t,
            "Переход к прямому расчету pt после обратного уравнения (p, h)"
        );
        self.calculate_pt(p, t)
    }

    #[instrument(level = "debug", skip(self))]
    fn calculate_ps(&self, p: f64, s: f64) -> Result<WaterState, If97Error> {
        let t = Self::calc_t_ps(p, s);
        if t < T_MIN_IF97 || t > 623.15 || p > P_MAX_IF97 {
            error!(t, "Вычисленная температура {}K вне границ Региона 1", t);
            return Err(If97Error::OutOfBounds(
                "Точка (p, s) лежит вне границ Региона 1".into(),
            ));
        }
        debug!(
            t,
            "Переход к прямому расчету pt после обратного уравнения (p, s)"
        );
        self.calculate_pt(p, t)
    }
}
