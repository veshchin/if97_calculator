// File: src/domain/models/region_5.rs

use crate::domain::state::{Region, WaterState};
use crate::domain::traits::WaterRegionModel;
use crate::domain::math::GibbsRegion;
use crate::domain::errors::If97Error;
use crate::domain::constants::R;
use crate::domain::tables::{REGION5, REGION5_CP0};
use tracing::{instrument, trace, debug, error};

pub struct Region5;

impl Region5 {
    const P_STAR: f64 = 1.0;
    const T_STAR: f64 = 1000.0;

    #[inline(always)]
    #[instrument(level = "trace")]
    fn precompute_pi_powers(pi: f64) -> [f64; 60] {
        let mut powers = [0.0; 60];
        for i in 0..60 { powers[i] = pi.powi(i as i32); }
        powers
    }

    #[inline(always)]
    #[instrument(level = "trace")]
    fn precompute_tau_powers(tau: f64) -> [f64; 60] {
        let mut powers = [0.0; 60];
        for j in 0..60 { powers[j] = tau.powi(j as i32); }
        powers
    }

    // --- 1D решатели для обратных расчетов ---
    #[instrument(level = "debug", skip(self))]
    fn calc_t_ph(&self, p: f64, h_target: f64) -> Result<f64, If97Error> {
        let pi = p / Self::P_STAR;
        let mut t = 1500.0;
        debug!(p, h_target, "Старт 1D решателя Region 5 (p, h)");

        for iter in 0..20 {
            let tau = Self::T_STAR / t;
            let gamma_tau = self.gamma_tau(pi, tau);
            let gamma_tau_tau = self.gamma_tau_tau(pi, tau);

            let h_curr = R * t * tau * gamma_tau;
            let cp_curr = -R * tau.powi(2) * gamma_tau_tau;

            let f = h_curr - h_target;

            trace!(iter, t, h_curr, f, cp_curr, "Итерация решателя (p, h)");

            if f.abs() < 1e-7 {
                debug!(iter, t, "Сходимость решателя (p, h) достигнута");
                return Ok(t);
            }
            t = (t - f / cp_curr).clamp(1073.15, 2273.15);
        }
        error!("Превышен лимит итераций в решателе Region 5 (p, h).");
        Err(If97Error::ConvergenceError("Превышен лимит итераций в решателе Region 5 (p, h).".into()))
    }

    #[instrument(level = "debug", skip(self))]
    fn calc_t_ps(&self, p: f64, s_target: f64) -> Result<f64, If97Error> {
        let pi = p / Self::P_STAR;
        let mut t = 1500.0;
        debug!(p, s_target, "Старт 1D решателя Region 5 (p, s)");

        for iter in 0..20 {
            let tau = Self::T_STAR / t;
            let gamma = self.gamma(pi, tau);
            let gamma_tau = self.gamma_tau(pi, tau);
            let gamma_tau_tau = self.gamma_tau_tau(pi, tau);

            let s_curr = R * (tau * gamma_tau - gamma);
            let cp_curr = -R * tau.powi(2) * gamma_tau_tau;

            let ds_dt = cp_curr / t;
            let f = s_curr - s_target;

            trace!(iter, t, s_curr, f, ds_dt, "Итерация решателя (p, s)");

            if f.abs() < 1e-7 {
                debug!(iter, t, "Сходимость решателя (p, s) достигнута");
                return Ok(t);
            }
            t = (t - f / ds_dt).clamp(1073.15, 2273.15);
        }
        error!("Превышен лимит итераций в решателе Region 5 (p, s).");
        Err(If97Error::ConvergenceError("Превышен лимит итераций в решателе Region 5 (p, s).".into()))
    }
}

impl GibbsRegion for Region5 {
    #[instrument(level = "trace", skip(self))]
    fn gamma(&self, pi: f64, tau: f64) -> f64 {
        let mut gamma_o = pi.ln();
        for &(n, j) in REGION5_CP0.iter() { gamma_o += n * tau.powi(j); }
        let pi_powers = Self::precompute_pi_powers(pi);
        let tau_powers = Self::precompute_tau_powers(tau);
        let gamma_r = REGION5.iter().fold(0.0, |acc, &(n, i, j)| acc + n * pi_powers[i as usize] * tau_powers[j as usize]);
        let res = gamma_o + gamma_r;
        trace!(res, "Рассчитана базовая gamma");
        res
    }

    #[instrument(level = "trace", skip(self))]
    fn gamma_pi(&self, pi: f64, tau: f64) -> f64 {
        let gamma_o_pi = 1.0 / pi;
        let pi_powers = Self::precompute_pi_powers(pi);
        let tau_powers = Self::precompute_tau_powers(tau);
        let gamma_r_pi = REGION5.iter().fold(0.0, |acc, &(n, i, j)| {
            if i == 0 { return acc; }
            acc + n * (i as f64) * pi_powers[(i - 1) as usize] * tau_powers[j as usize]
        });
        let res = gamma_o_pi + gamma_r_pi;
        trace!(res, "Рассчитана производная gamma_pi");
        res
    }

    #[instrument(level = "trace", skip(self))]
    fn gamma_tau(&self, pi: f64, tau: f64) -> f64 {
        let mut gamma_o_tau = 0.0;
        for &(n, j) in REGION5_CP0.iter() {
            if j != 0 { gamma_o_tau += n * (j as f64) * tau.powi(j - 1); }
        }
        let pi_powers = Self::precompute_pi_powers(pi);
        let tau_powers = Self::precompute_tau_powers(tau);
        let gamma_r_tau = REGION5.iter().fold(0.0, |acc, &(n, i, j)| {
            if j == 0 { return acc; }
            acc + n * pi_powers[i as usize] * (j as f64) * tau_powers[(j - 1) as usize]
        });
        let res = gamma_o_tau + gamma_r_tau;
        trace!(res, "Рассчитана производная gamma_tau");
        res
    }

    #[instrument(level = "trace", skip(self))]
    fn gamma_pi_pi(&self, pi: f64, tau: f64) -> f64 {
        let gamma_o_pi_pi = -1.0 / (pi * pi);
        let pi_powers = Self::precompute_pi_powers(pi);
        let tau_powers = Self::precompute_tau_powers(tau);
        let gamma_r_pi_pi = REGION5.iter().fold(0.0, |acc, &(n, i, j)| {
            if i <= 1 { return acc; }
            acc + n * (i as f64) * ((i - 1) as f64) * pi_powers[(i - 2) as usize] * tau_powers[j as usize]
        });
        let res = gamma_o_pi_pi + gamma_r_pi_pi;
        trace!(res, "Рассчитана производная gamma_pi_pi");
        res
    }

    #[instrument(level = "trace", skip(self))]
    fn gamma_tau_tau(&self, pi: f64, tau: f64) -> f64 {
        let mut gamma_o_tau_tau = 0.0;
        for &(n, j) in REGION5_CP0.iter() {
            if j != 0 && j != 1 { gamma_o_tau_tau += n * (j as f64) * ((j - 1) as f64) * tau.powi(j - 2); }
        }
        let pi_powers = Self::precompute_pi_powers(pi);
        let tau_powers = Self::precompute_tau_powers(tau);
        let gamma_r_tau_tau = REGION5.iter().fold(0.0, |acc, &(n, i, j)| {
            if j <= 1 { return acc; }
            acc + n * pi_powers[i as usize] * (j as f64) * ((j - 1) as f64) * tau_powers[(j - 2) as usize]
        });
        let res = gamma_o_tau_tau + gamma_r_tau_tau;
        trace!(res, "Рассчитана производная gamma_tau_tau");
        res
    }

    #[instrument(level = "trace", skip(self))]
    fn gamma_pi_tau(&self, pi: f64, tau: f64) -> f64 {
        let pi_powers = Self::precompute_pi_powers(pi);
        let tau_powers = Self::precompute_tau_powers(tau);
        let res = REGION5.iter().fold(0.0, |acc, &(n, i, j)| {
            if i == 0 || j == 0 { return acc; }
            acc + n * (i as f64) * pi_powers[(i - 1) as usize] * (j as f64) * tau_powers[(j - 1) as usize]
        });
        trace!(res, "Рассчитана смешанная производная gamma_pi_tau");
        res
    }
}

impl WaterRegionModel for Region5 {
    #[instrument(level = "debug", skip(self))]
    fn calculate_pt(&self, p: f64, t: f64) -> Result<WaterState, If97Error> {
        let pi = p / Self::P_STAR;
        let tau = Self::T_STAR / t;
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

        let w_squared = (R * t * gamma_pi.powi(2)) /
            ( (gamma_pi - tau * gamma_pi_tau).powi(2) / (tau.powi(2) * gamma_tau_tau) - gamma_pi_pi );
        let w = if w_squared > 0.0 { (w_squared * 1000.0).sqrt() } else { f64::NAN };

        debug!(v, h, s, cp, w, "Успешный прямой расчет свойств Region5 (p, t)");
        Ok(WaterState { p, t, v, rho: 1.0 / v, h, s, cp, w, region: Region::Region5 })
    }

    #[instrument(level = "debug", skip(self))]
    fn calculate_ph(&self, p: f64, h: f64) -> Result<WaterState, If97Error> {
        let t = self.calc_t_ph(p, h)?;
        if t < 1073.15 || t > 2273.15 || p < 0.0 || p > 50.0 {
            error!(t, p, "Вычисленные параметры (p, t) вне границ Региона 5");
            return Err(If97Error::OutOfBounds("Точка (p, h) лежит вне границ Региона 5".into()));
        }
        debug!(t, "Успешно получена температура из решателя (p, h), переход к прямому расчету");
        self.calculate_pt(p, t)
    }

    #[instrument(level = "debug", skip(self))]
    fn calculate_ps(&self, p: f64, s: f64) -> Result<WaterState, If97Error> {
        let t = self.calc_t_ps(p, s)?;
        if t < 1073.15 || t > 2273.15 || p < 0.0 || p > 50.0 {
            error!(t, p, "Вычисленные параметры (p, t) вне границ Региона 5");
            return Err(If97Error::OutOfBounds("Точка (p, s) лежит вне границ Региона 5".into()));
        }
        debug!(t, "Успешно получена температура из решателя (p, s), переход к прямому расчету");
        self.calculate_pt(p, t)
    }
}