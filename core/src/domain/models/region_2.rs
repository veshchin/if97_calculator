// File: src/domain/models/region_2.rs

use crate::domain::state::{Region, WaterState};
use crate::domain::traits::WaterRegionModel;
use crate::domain::math::GibbsRegion;
use crate::domain::errors::If97Error;
use crate::domain::constants::R;
use crate::domain::boundaries::{Region2Subregion, determine_region2_subregion_ph, determine_region2_subregion_ps};
use crate::domain::tables::{
    REGION2, REGION2_CP0,
    BACKWARD2A_T_PH, BACKWARD2B_T_PH, BACKWARD2C_T_PH,
    BACKWARD2A_T_PS, BACKWARD2B_T_PS, BACKWARD2C_T_PS
};
use tracing::{instrument, trace, debug, error};

pub struct Region2;

impl Region2 {
    const P_STAR: f64 = 1.0;
    const T_STAR: f64 = 540.0;

    #[inline(always)]
    #[instrument(level = "trace")]
    fn precompute_pi_powers(pi: f64) -> [f64; 60] {
        let mut powers = [0.0; 60];
        for i in 0..60 { powers[i] = pi.powi(i as i32); }
        powers
    }

    #[inline(always)]
    #[instrument(level = "trace")]
    fn precompute_tau_powers(tau_term: f64) -> [f64; 60] {
        let mut powers = [0.0; 60];
        for j in 0..60 { powers[j] = tau_term.powi(j as i32); }
        powers
    }

    // --- Обратные формулы ---
    #[instrument(level = "trace")]
    fn calc_t_ph(p: f64, h: f64) -> f64 {
        let t = match determine_region2_subregion_ph(p, h) {
            Region2Subregion::Region2a => {
                let mut t_val = 0.0;
                for &(n, i, j) in BACKWARD2A_T_PH.iter() {
                    t_val += n * p.powi(i) * ((h / 2000.0) - 2.1).powi(j);
                }
                trace!("Использовано уравнение субрегиона 2a");
                t_val
            },
            Region2Subregion::Region2b => {
                let mut t_val = 0.0;
                for &(n, i, j) in BACKWARD2B_T_PH.iter() {
                    t_val += n * (p - 2.0).powi(i) * ((h / 2000.0) - 2.6).powi(j);
                }
                trace!("Использовано уравнение субрегиона 2b");
                t_val
            },
            Region2Subregion::Region2c => {
                let mut t_val = 0.0;
                for &(n, i, j) in BACKWARD2C_T_PH.iter() {
                    t_val += n * (p + 25.0).powi(i) * ((h / 2000.0) - 1.8).powi(j);
                }
                trace!("Использовано уравнение субрегиона 2c");
                t_val
            },
        };
        trace!(t, "Вычислена температура по обратному уравнению (p, h) для Region2");
        t
    }

    #[instrument(level = "trace")]
    fn calc_t_ps(p: f64, s: f64) -> f64 {
        let t = match determine_region2_subregion_ps(p, s) {
            Region2Subregion::Region2a => {
                let mut t_val = 0.0;
                for &(n, i, j) in BACKWARD2A_T_PS.iter() {
                    t_val += n * p.powf(i as f64) * ((s / 2.0) - 2.0).powi(j);
                }
                trace!("Использовано уравнение субрегиона 2a");
                t_val
            },
            Region2Subregion::Region2b => {
                let mut t_val = 0.0;
                for &(n, i, j) in BACKWARD2B_T_PS.iter() {
                    t_val += n * p.powi(i) * (10.0 - (s / 0.7853)).powi(j);
                }
                trace!("Использовано уравнение субрегиона 2b");
                t_val
            },
            Region2Subregion::Region2c => {
                let mut t_val = 0.0;
                for &(n, i, j) in BACKWARD2C_T_PS.iter() {
                    t_val += n * p.powi(i) * (2.0 - (s / 2.9251)).powi(j);
                }
                trace!("Использовано уравнение субрегиона 2c");
                t_val
            },
        };
        trace!(t, "Вычислена температура по обратному уравнению (p, s) для Region2");
        t
    }
}

impl GibbsRegion for Region2 {
    #[instrument(level = "trace", skip(self))]
    fn gamma(&self, pi: f64, tau: f64) -> f64 {
        let mut gamma_o = pi.ln();
        for &(n, j) in REGION2_CP0.iter() { gamma_o += n * tau.powi(j); }
        let tau_term = tau - 0.5;
        let pi_powers = Self::precompute_pi_powers(pi);
        let tau_powers = Self::precompute_tau_powers(tau_term);
        let gamma_r = REGION2.iter().fold(0.0, |acc, &(n, i, j)| acc + n * pi_powers[i as usize] * tau_powers[j as usize]);
        let res = gamma_o + gamma_r;
        trace!(res, "Рассчитана базовая gamma");
        res
    }

    #[instrument(level = "trace", skip(self))]
    fn gamma_pi(&self, pi: f64, tau: f64) -> f64 {
        let gamma_o_pi = 1.0 / pi;
        let tau_term = tau - 0.5;
        let pi_powers = Self::precompute_pi_powers(pi);
        let tau_powers = Self::precompute_tau_powers(tau_term);
        let gamma_r_pi = REGION2.iter().fold(0.0, |acc, &(n, i, j)| {
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
        for &(n, j) in REGION2_CP0.iter() {
            if j != 0 { gamma_o_tau += n * (j as f64) * tau.powi(j - 1); }
        }
        let tau_term = tau - 0.5;
        let pi_powers = Self::precompute_pi_powers(pi);
        let tau_powers = Self::precompute_tau_powers(tau_term);
        let gamma_r_tau = REGION2.iter().fold(0.0, |acc, &(n, i, j)| {
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
        let tau_term = tau - 0.5;
        let pi_powers = Self::precompute_pi_powers(pi);
        let tau_powers = Self::precompute_tau_powers(tau_term);
        let gamma_r_pi_pi = REGION2.iter().fold(0.0, |acc, &(n, i, j)| {
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
        for &(n, j) in REGION2_CP0.iter() {
            if j != 0 && j != 1 { gamma_o_tau_tau += n * (j as f64) * ((j - 1) as f64) * tau.powi(j - 2); }
        }
        let tau_term = tau - 0.5;
        let pi_powers = Self::precompute_pi_powers(pi);
        let tau_powers = Self::precompute_tau_powers(tau_term);
        let gamma_r_tau_tau = REGION2.iter().fold(0.0, |acc, &(n, i, j)| {
            if j <= 1 { return acc; }
            acc + n * pi_powers[i as usize] * (j as f64) * ((j - 1) as f64) * tau_powers[(j - 2) as usize]
        });
        let res = gamma_o_tau_tau + gamma_r_tau_tau;
        trace!(res, "Рассчитана производная gamma_tau_tau");
        res
    }

    #[instrument(level = "trace", skip(self))]
    fn gamma_pi_tau(&self, pi: f64, tau: f64) -> f64 {
        let tau_term = tau - 0.5;
        let pi_powers = Self::precompute_pi_powers(pi);
        let tau_powers = Self::precompute_tau_powers(tau_term);
        let res = REGION2.iter().fold(0.0, |acc, &(n, i, j)| {
            if i == 0 || j == 0 { return acc; }
            acc + n * (i as f64) * pi_powers[(i - 1) as usize] * (j as f64) * tau_powers[(j - 1) as usize]
        });
        trace!(res, "Рассчитана смешанная производная gamma_pi_tau");
        res
    }
}

impl WaterRegionModel for Region2 {
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

        debug!(v, h, s, cp, w, "Успешный прямой расчет свойств Region2 (p, t)");
        Ok(WaterState { p, t, v, rho: 1.0 / v, h, s, cp, w, region: Region::Region2 })
    }

    #[instrument(level = "debug", skip(self))]
    fn calculate_ph(&self, p: f64, h: f64) -> Result<WaterState, If97Error> {
        let t = Self::calc_t_ph(p, h);
        if t < 273.15 || t > 1073.15 || p > 100.0 {
            error!(t, "Вычисленная температура {}K вне границ Региона 2", t);
            return Err(If97Error::OutOfBounds("Точка (p, h) лежит вне границ Региона 2".into()));
        }
        debug!(t, "Переход к прямому расчету pt после обратного уравнения (p, h)");
        self.calculate_pt(p, t)
    }

    #[instrument(level = "debug", skip(self))]
    fn calculate_ps(&self, p: f64, s: f64) -> Result<WaterState, If97Error> {
        let t = Self::calc_t_ps(p, s);
        if t < 273.15 || t > 1073.15 || p > 100.0 {
            error!(t, "Вычисленная температура {}K вне границ Региона 2", t);
            return Err(If97Error::OutOfBounds("Точка (p, s) лежит вне границ Региона 2".into()));
        }
        debug!(t, "Переход к прямому расчету pt после обратного уравнения (p, s)");
        self.calculate_pt(p, t)
    }
}