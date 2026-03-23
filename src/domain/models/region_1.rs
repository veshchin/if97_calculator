use crate::math::ThermodynamicRegion;
use crate::tables::REGION1;

pub struct Region1;

impl Region1 {
    #[inline(always)]
    fn precompute_pi_powers(pi_term: f64) -> [f64; 35] {
        let mut powers = [0.0; 35];
        for i in 0..35 { powers[i] = pi_term.powi(i as i32); }
        powers
    }

    // Увеличен массив до 61 элемента, чтобы вместить j-2 (от -43 до +17)
    #[inline(always)]
    fn precompute_tau_powers(tau_term: f64) -> [f64; 61] {
        let mut powers = [0.0; 61];
        for j in -43..=17 {
            powers[(j + 43) as usize] = tau_term.powi(j as i32);
        }
        powers
    }
}

impl ThermodynamicRegion for Region1 {
    fn gamma(&self, pi: f64, tau: f64) -> f64 {
        let pi_term = 7.1 - pi;
        let tau_term = tau - 1.222;
        let pi_powers = Self::precompute_pi_powers(pi_term);
        let tau_powers = Self::precompute_tau_powers(tau_term);

        REGION1.iter().fold(0.0, |acc, &(n, i, j)| {
            acc + n * pi_powers[i as usize] * tau_powers[(j + 43) as usize]
        })
    }

    fn gamma_pi(&self, pi: f64, tau: f64) -> f64 {
        let pi_term = 7.1 - pi;
        let tau_term = tau - 1.222;
        let pi_powers = Self::precompute_pi_powers(pi_term);
        let tau_powers = Self::precompute_tau_powers(tau_term);

        REGION1.iter().fold(0.0, |acc, &(n, i, j)| {
            if i == 0 { return acc; }
            acc - n * (i as f64) * pi_powers[(i - 1) as usize] * tau_powers[(j + 43) as usize]
        })
    }

    fn gamma_tau(&self, pi: f64, tau: f64) -> f64 {
        let pi_term = 7.1 - pi;
        let tau_term = tau - 1.222;
        let pi_powers = Self::precompute_pi_powers(pi_term);
        let tau_powers = Self::precompute_tau_powers(tau_term);

        REGION1.iter().fold(0.0, |acc, &(n, i, j)| {
            if j == 0 { return acc; }
            acc + n * pi_powers[i as usize] * (j as f64) * tau_powers[(j - 1 + 43) as usize]
        })
    }

    fn gamma_pi_pi(&self, pi: f64, tau: f64) -> f64 {
        let pi_term = 7.1 - pi;
        let tau_term = tau - 1.222;
        let pi_powers = Self::precompute_pi_powers(pi_term);
        let tau_powers = Self::precompute_tau_powers(tau_term);

        REGION1.iter().fold(0.0, |acc, &(n, i, j)| {
            if i <= 1 { return acc; }
            acc + n * (i as f64) * ((i - 1) as f64) * pi_powers[(i - 2) as usize] * tau_powers[(j + 43) as usize]
        })
    }

    fn gamma_tau_tau(&self, pi: f64, tau: f64) -> f64 {
        let pi_term = 7.1 - pi;
        let tau_term = tau - 1.222;
        let pi_powers = Self::precompute_pi_powers(pi_term);
        let tau_powers = Self::precompute_tau_powers(tau_term);

        REGION1.iter().fold(0.0, |acc, &(n, i, j)| {
            if j == 0 || j == 1 { return acc; }
            acc + n * pi_powers[i as usize] * (j as f64) * ((j - 1) as f64) * tau_powers[(j - 2 + 43) as usize]
        })
    }

    fn gamma_pi_tau(&self, pi: f64, tau: f64) -> f64 {
        let pi_term = 7.1 - pi;
        let tau_term = tau - 1.222;
        let pi_powers = Self::precompute_pi_powers(pi_term);
        let tau_powers = Self::precompute_tau_powers(tau_term);

        REGION1.iter().fold(0.0, |acc, &(n, i, j)| {
            if i == 0 || j == 0 { return acc; }
            acc - n * (i as f64) * pi_powers[(i - 1) as usize] * (j as f64) * tau_powers[(j - 1 + 43) as usize]
        })
    }
}