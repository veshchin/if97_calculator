// File: src/domain/models/region_2_meta.rs

use crate::domain::constants::R;
use crate::domain::state::{Region, WaterState};
use crate::domain::traits::WaterRegionModel;
use crate::domain::math::GibbsRegion;
use crate::domain::errors::If97Error;
use crate::domain::tables::{REGION2_META, REGION2_CP0};
use tracing::{instrument, trace, debug};

pub struct Region2Meta;

impl Region2Meta {
    const P_STAR: f64 = 1.0;
    const T_STAR: f64 = 540.0;
    const N1_0: f64 = -0.96937268393049e1;
    const N2_0: f64 = 0.10087275970006e2;

    #[inline(always)]
    #[instrument(level = "trace")]
    fn precompute_pi_powers(pi: f64) -> [f64; 60] {
        let mut powers = [0.0; 60];
        powers[0] = 1.0;
        for i in 1..60 { powers[i] = powers[i - 1] * pi; }
        powers
    }

    #[inline(always)]
    #[instrument(level = "trace")]
    fn precompute_tau_powers(tau_term: f64) -> [f64; 60] {
        let mut powers = [0.0; 60];
        powers[0] = 1.0;
        for j in 1..60 { powers[j] = powers[j - 1] * tau_term; }
        powers
    }
}

impl GibbsRegion for Region2Meta {
    #[instrument(level = "trace", skip(self))]
    fn gamma(&self, pi: f64, tau: f64) -> f64 {
        let mut gamma_o = pi.ln() + Self::N1_0 + Self::N2_0 * tau;
        for &(n, j) in REGION2_CP0.iter().skip(2) {
            gamma_o += n * tau.powi(j as i32);
        }

        let pi_powers = Self::precompute_pi_powers(pi);
        let tau_term = tau - 0.5;
        let tau_powers = Self::precompute_tau_powers(tau_term);
        let gamma_r = REGION2_META.iter().fold(0.0, |acc, &(n, i, j)| {
            acc + n * pi_powers[i as usize] * tau_powers[j as usize]
        });
        let res = gamma_o + gamma_r;
        trace!(res, "Рассчитана базовая gamma метастабильного пара");
        res
    }

    #[instrument(level = "trace", skip(self))]
    fn gamma_tau(&self, pi: f64, tau: f64) -> f64 {
        let mut gamma_o_tau = Self::N2_0;
        for &(n, j) in REGION2_CP0.iter().skip(2) {
            if j != 0 { gamma_o_tau += n * (j as f64) * tau.powi((j - 1) as i32); }
        }

        let tau_term = tau - 0.5;
        let pi_powers = Self::precompute_pi_powers(pi);
        let tau_powers = Self::precompute_tau_powers(tau_term);
        let gamma_r_tau = REGION2_META.iter().fold(0.0, |acc, &(n, i, j)| {
            if j == 0 { return acc; }
            acc + n * pi_powers[i as usize] * (j as f64) * tau_powers[(j - 1) as usize]
        });
        let res = gamma_o_tau + gamma_r_tau;
        trace!(res, "Рассчитана производная gamma_tau метастабильного пара");
        res
    }

    #[instrument(level = "trace", skip(self))]
    fn gamma_pi(&self, pi: f64, tau: f64) -> f64 {
        let gamma_o_pi = 1.0 / pi;
        let pi_powers = Self::precompute_pi_powers(pi);

        let tau_term = tau - 0.5;
        let tau_powers = Self::precompute_tau_powers(tau_term);
        let gamma_r_pi = REGION2_META.iter().fold(0.0, |acc, &(n, i, j)| {
            if i == 0 { return acc; }
            acc + n * (i as f64) * pi_powers[(i - 1) as usize] * tau_powers[j as usize]
        });
        let res = gamma_o_pi + gamma_r_pi;
        trace!(res, "Рассчитана производная gamma_pi метастабильного пара");
        res
    }

    #[instrument(level = "trace", skip(self))]
    fn gamma_pi_pi(&self, pi: f64, tau: f64) -> f64 {
        let gamma_o_pi_pi = -1.0 / (pi * pi);
        let pi_powers = Self::precompute_pi_powers(pi);

        let tau_term = tau - 0.5;
        let tau_powers = Self::precompute_tau_powers(tau_term);
        let gamma_r_pi_pi = REGION2_META.iter().fold(0.0, |acc, &(n, i, j)| {
            if i <= 1 { return acc; }
            acc + n * (i as f64) * ((i - 1) as f64) * pi_powers[(i - 2) as usize] * tau_powers[j as usize]
        });
        let res = gamma_o_pi_pi + gamma_r_pi_pi;
        trace!(res, "Рассчитана производная gamma_pi_pi метастабильного пара");
        res
    }

    #[instrument(level = "trace", skip(self))]
    fn gamma_tau_tau(&self, pi: f64, tau: f64) -> f64 {
        let mut gamma_o_tau_tau = 0.0;
        for &(n, j) in REGION2_CP0.iter().skip(2) {
            if j != 0 && j != 1 {
                gamma_o_tau_tau += n * (j as f64) * ((j - 1) as f64) * tau.powi((j - 2) as i32);
            }
        }

        let tau_term = tau - 0.5;
        let pi_powers = Self::precompute_pi_powers(pi);
        let tau_powers = Self::precompute_tau_powers(tau_term);
        let gamma_r_tau_tau = REGION2_META.iter().fold(0.0, |acc, &(n, i, j)| {
            if j <= 1 { return acc; }
            acc + n * pi_powers[i as usize] * (j as f64) * ((j - 1) as f64) * tau_powers[(j - 2) as usize]
        });
        let res = gamma_o_tau_tau + gamma_r_tau_tau;
        trace!(res, "Рассчитана производная gamma_tau_tau метастабильного пара");
        res
    }

    #[instrument(level = "trace", skip(self))]
    fn gamma_pi_tau(&self, pi: f64, tau: f64) -> f64 {
        let pi_powers = Self::precompute_pi_powers(pi);
        let tau_term = tau - 0.5;
        let tau_powers = Self::precompute_tau_powers(tau_term);
        let res = REGION2_META.iter().fold(0.0, |acc, &(n, i, j)| {
            if i == 0 || j == 0 { return acc; }
            acc + n * (i as f64) * pi_powers[(i - 1) as usize] * (j as f64) * tau_powers[(j - 1) as usize]
        });
        trace!(res, "Рассчитана смешанная производная gamma_pi_tau метастабильного пара");
        res
    }
}

impl WaterRegionModel for Region2Meta {
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

        debug!(v, h, s, cp, w, "Успешный прямой расчет свойств Region2Meta (p, t)");
        Ok(WaterState { p, t, v, rho: 1.0 / v, h, s, cp, w, region: Region::Region2 })
    }
}