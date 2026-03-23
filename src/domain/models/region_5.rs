use crate::math::ThermodynamicRegion;
use crate::tables::{REGION5, REGION5_CP0};

pub struct Region5;

impl Region5 {
    #[inline(always)]
    fn precompute_pi_powers(pi: f64) -> [f64; 60] {
        let mut powers = [0.0; 60];
        for i in 0..60 { powers[i] = pi.powi(i as i32); }
        powers
    }

    #[inline(always)]
    fn precompute_tau_powers(tau: f64) -> [f64; 60] {
        let mut powers = [0.0; 60];
        for j in 0..60 { powers[j] = tau.powi(j as i32); }
        powers
    }
}

impl ThermodynamicRegion for Region5 {
    fn gamma(&self, pi: f64, tau: f64) -> f64 {
        let mut gamma_o = pi.ln();
        for &(n, j) in REGION5_CP0.iter() {
            gamma_o += n * tau.powi(j);
        }

        let pi_powers = Self::precompute_pi_powers(pi);
        let tau_powers = Self::precompute_tau_powers(tau);

        let gamma_r = REGION5.iter().fold(0.0, |acc, &(n, i, j)| {
            acc + n * pi_powers[i as usize] * tau_powers[j as usize]
        });
        gamma_o + gamma_r
    }

    fn gamma_pi(&self, pi: f64, tau: f64) -> f64 {
        let gamma_o_pi = 1.0 / pi;
        let pi_powers = Self::precompute_pi_powers(pi);
        let tau_powers = Self::precompute_tau_powers(tau);

        let gamma_r_pi = REGION5.iter().fold(0.0, |acc, &(n, i, j)| {
            if i == 0 { return acc; }
            acc + n * (i as f64) * pi_powers[(i - 1) as usize] * tau_powers[j as usize]
        });
        gamma_o_pi + gamma_r_pi
    }

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
        gamma_o_tau + gamma_r_tau
    }

    fn gamma_pi_pi(&self, pi: f64, tau: f64) -> f64 {
        let gamma_o_pi_pi = -1.0 / (pi * pi);
        let pi_powers = Self::precompute_pi_powers(pi);
        let tau_powers = Self::precompute_tau_powers(tau);

        let gamma_r_pi_pi = REGION5.iter().fold(0.0, |acc, &(n, i, j)| {
            if i <= 1 { return acc; }
            acc + n * (i as f64) * ((i - 1) as f64) * pi_powers[(i - 2) as usize] * tau_powers[j as usize]
        });
        gamma_o_pi_pi + gamma_r_pi_pi
    }

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
        gamma_o_tau_tau + gamma_r_tau_tau
    }

    fn gamma_pi_tau(&self, pi: f64, tau: f64) -> f64 {
        let pi_powers = Self::precompute_pi_powers(pi);
        let tau_powers = Self::precompute_tau_powers(tau);

        REGION5.iter().fold(0.0, |acc, &(n, i, j)| {
            if i == 0 || j == 0 { return acc; }
            acc + n * (i as f64) * pi_powers[(i - 1) as usize] * (j as f64) * tau_powers[(j - 1) as usize]
        })
    }
}
