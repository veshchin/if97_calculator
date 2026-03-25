use crate::domain::state::{Region, WaterState};
use crate::domain::traits::WaterRegionModel;
use crate::domain::constants::R;
use crate::domain::tables::{REGION5, REGION5_CP0};

pub struct Region5;

impl Region5 {
    const P_STAR: f64 = 1.0;
    const T_STAR: f64 = 1000.0;

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

    // --- Математика (Гамма-функции) ---
    fn gamma(&self, pi: f64, tau: f64) -> f64 {
        let mut gamma_o = pi.ln();
        for &(n, j) in REGION5_CP0.iter() { gamma_o += n * tau.powi(j); }
        let pi_powers = Self::precompute_pi_powers(pi);
        let tau_powers = Self::precompute_tau_powers(tau);
        let gamma_r = REGION5.iter().fold(0.0, |acc, &(n, i, j)| acc + n * pi_powers[i as usize] * tau_powers[j as usize]);
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

    // --- 1D решатели для обратных расчетов ---
    fn calc_t_ph(&self, p: f64, h_target: f64) -> Result<f64, &'static str> {
        let pi = p / Self::P_STAR;
        let mut t = 1500.0;
        for _ in 0..20 {
            let tau = Self::T_STAR / t;
            let gamma_tau = self.gamma_tau(pi, tau);
            let gamma_tau_tau = self.gamma_tau_tau(pi, tau);
            let h_curr = R * t * tau * gamma_tau;
            let cp_curr = -R * tau.powi(2) * gamma_tau_tau;
            let f = h_curr - h_target;
            if f.abs() < 1e-7 { return Ok(t); }
            t = (t - f / cp_curr).clamp(1073.15, 2273.15);
        }
        Err("Превышен лимит итераций в решателе Region 5 (p, h).")
    }

    fn calc_t_ps(&self, p: f64, s_target: f64) -> Result<f64, &'static str> {
        let pi = p / Self::P_STAR;
        let mut t = 1500.0;
        for _ in 0..20 {
            let tau = Self::T_STAR / t;
            let gamma = self.gamma(pi, tau);
            let gamma_tau = self.gamma_tau(pi, tau);
            let gamma_tau_tau = self.gamma_tau_tau(pi, tau);
            let s_curr = R * (tau * gamma_tau - gamma);
            let cp_curr = -R * tau.powi(2) * gamma_tau_tau;
            let ds_dt = cp_curr / t;
            let f = s_curr - s_target;
            if f.abs() < 1e-7 { return Ok(t); }
            t = (t - f / ds_dt).clamp(1073.15, 2273.15);
        }
        Err("Превышен лимит итераций в решателе Region 5 (p, s).")
    }
}

impl WaterRegionModel for Region5 {
    fn calculate_pt(&self, p: f64, t: f64) -> Result<WaterState, &'static str> {
        let pi = p / Self::P_STAR;
        let tau = Self::T_STAR / t;

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

        Ok(WaterState { p, t, v, rho: 1.0 / v, h, s, cp, w, region: Region::Region5 })
    }

    fn calculate_ph(&self, p: f64, h: f64) -> Result<WaterState, &'static str> {
        let t = self.calc_t_ph(p, h)?;
        if t < 1073.15 || t > 2273.15 || p < 0.0 || p > 50.0 {
            return Err("Точка (p, h) лежит вне границ Региона 5");
        }
        self.calculate_pt(p, t)
    }

    fn calculate_ps(&self, p: f64, s: f64) -> Result<WaterState, &'static str> {
        let t = self.calc_t_ps(p, s)?;
        if t < 1073.15 || t > 2273.15 || p < 0.0 || p > 50.0 {
            return Err("Точка (p, s) лежит вне границ Региона 5");
        }
        self.calculate_pt(p, t)
    }
}