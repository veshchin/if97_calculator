// File: src/domain/models/region_3.rs

use crate::domain::state::{Region, WaterState};
use crate::domain::traits::WaterRegionModel;
use crate::domain::math::HelmholtzRegion;
use crate::domain::errors::If97Error;
use crate::domain::constants::*;
use crate::domain::tables::REGION3;
use tracing::{instrument, trace, debug, warn, error};

const P_MIN: f64 = 16.0;
const P_MAX: f64 = 100.0;
const P_NODES: usize = 4;
const H_MIN: f64 = 1600.0;
const H_MAX: f64 = 3800.0;
const H_NODES: usize = 4;
const S_MIN: f64 = 3.8;
const S_MAX: f64 = 6.4;
const S_NODES: usize = 4;

const GRID_PH: [[(f64, f64); 4]; 4] = [
    [(850.0, 625.0), (600.0, 635.0), (200.0, 650.0), ( 20.0, 750.0)],
    [(870.0, 635.0), (650.0, 680.0), (350.0, 710.0), ( 40.0, 800.0)],
    [(900.0, 645.0), (700.0, 710.0), (480.0, 760.0), ( 80.0, 840.0)],
    [(920.0, 650.0), (730.0, 730.0), (550.0, 780.0), (100.0, 860.0)],
];

const GRID_PS: [[(f64, f64); 4]; 4] = [
    [(850.0, 625.0), (520.0, 635.0), (150.0, 650.0), ( 25.0, 750.0)],
    [(870.0, 635.0), (620.0, 670.0), (320.0, 710.0), ( 55.0, 800.0)],
    [(900.0, 645.0), (690.0, 710.0), (450.0, 760.0), ( 85.0, 840.0)],
    [(920.0, 650.0), (740.0, 730.0), (550.0, 790.0), (110.0, 860.0)],
];

pub struct Region3;

impl Region3 {
    const N1: f64 = 1.0658070028513;

    #[inline(always)]
    #[instrument(level = "trace")]
    fn precompute_delta_powers(delta: f64) -> [f64; 15] {
        let mut powers = [0.0; 15];
        powers[0] = 1.0;
        for i in 1..15 { powers[i] = powers[i - 1] * delta; }
        powers
    }

    #[inline(always)]
    #[instrument(level = "trace")]
    fn precompute_tau_powers(tau: f64) -> [f64; 40] {
        let mut powers = [0.0; 40];
        powers[0] = 1.0;
        for j in 1..40 { powers[j] = powers[j - 1] * tau; }
        powers
    }

    #[instrument(level = "trace", skip(self))]
    fn calculate_density(&self, p: f64, t: f64, mut rho_guess: f64) -> f64 {
        let tau = T_C / t;
        let is_liquid = rho_guess > RHO_C;
        let mut delta = rho_guess / RHO_C;

        debug!(p, t, rho_guess, is_liquid, "Старт 1D решателя плотности (Region 3)");

        for iter in 0..SOLVER_MAX_ITER_DENSITY {
            let pd = self.phi_delta(delta, tau);
            let p_calc = rho_guess * R * t * delta * pd / 1000.0;
            let error = p_calc - p;

            trace!(iter, rho_guess, p_calc, error, "Итерация 1D решателя");

            if error.abs() < SOLVER_TOLERANCE_TIGHT {
                debug!(iter, rho_final = rho_guess, "Сходимость 1D решателя достигнута");
                break;
            }

            let pdd = self.phi_delta_delta(delta, tau);
            let dp_drho = R * t * (2.0 * delta * pd + delta.powi(2) * pdd) / 1000.0;

            if dp_drho <= 0.0 {
                warn!(iter, dp_drho, rho_guess, "Срабатывание защиты от спинодали, выталкивание плотности");
                rho_guess = if is_liquid { rho_guess + 20.0 } else { (rho_guess - 20.0).max(10.0) };
                delta = rho_guess / RHO_C;
                continue;
            }

            let mut step = error / dp_drho;
            let max_step = 0.05 * rho_guess;
            step = step.clamp(-max_step, max_step);

            let mut next_rho = rho_guess - step;

            if is_liquid && next_rho <= RHO_C {
                trace!("Защита фазы: ограничение снижения плотности жидкости");
                next_rho = RHO_C + 0.1;
            } else if !is_liquid && next_rho >= RHO_C {
                trace!("Защита фазы: ограничение роста плотности пара");
                next_rho = RHO_C - 0.1;
            }

            rho_guess = next_rho;
            delta = rho_guess / RHO_C;
        }
        rho_guess
    }

    #[inline(always)]
    #[instrument(level = "trace")]
    fn guess_rho_t_ph(p: f64, h: f64) -> (f64, f64) {
        let p_f = ((p - P_MIN) / (P_MAX - P_MIN)) * ((P_NODES - 1) as f64);
        let h_f = ((h - H_MIN) / (H_MAX - H_MIN)) * ((H_NODES - 1) as f64);
        let p_idx = p_f.clamp(0.0, (P_NODES - 1) as f64).round() as usize;
        let h_idx = h_f.clamp(0.0, (H_NODES - 1) as f64).round() as usize;
        let guess = GRID_PH[p_idx][h_idx];
        trace!(p_idx, h_idx, ?guess, "Получено начальное приближение (rho, t) из GRID_PH");
        guess
    }

    #[instrument(level = "trace", skip(self))]
    pub fn calculate_rho_t_ph(&self, p_target: f64, h_target: f64) -> Result<(f64, f64), If97Error> {
        let (initial_rho, initial_t) = Self::guess_rho_t_ph(p_target, h_target);

        let fallback_guesses = [
            (initial_rho, initial_t),
            (initial_rho * 1.05, initial_t + 5.0),
            (initial_rho * 0.95, initial_t - 5.0),
        ];

        debug!(p_target, h_target, ?fallback_guesses, "Старт 2D решателя (p, h) в Region 3");

        for (fallback_idx, &(start_rho, start_t)) in fallback_guesses.iter().enumerate() {
            let mut rho = start_rho;
            let mut t = start_t;
            let mut success = false;

            trace!(fallback_idx, rho, t, "Попытка решения с текущим начальным приближением");

            for iter in 0..SOLVER_MAX_ITER_2D {
                let delta = rho / RHO_C;
                let tau = T_C / t;
                let phi_d = self.phi_delta(delta, tau);
                let phi_t = self.phi_tau(delta, tau);
                let phi_dd = self.phi_delta_delta(delta, tau);
                let phi_tt = self.phi_tau_tau(delta, tau);
                let phi_dt = self.phi_delta_tau(delta, tau);

                let p_curr = rho * R * t * delta * phi_d / 1000.0;
                let h_curr = R * t * (tau * phi_t + delta * phi_d);
                let f1 = p_curr - p_target;
                let f2 = h_curr - h_target;

                trace!(iter, p_curr, h_curr, f1, f2, "Итерация 2D решателя (p, h)");

                if f1.abs() < SOLVER_TOLERANCE && f2.abs() < SOLVER_TOLERANCE {
                    debug!(iter, rho, t, "Сходимость 2D решателя (p, h) достигнута");
                    success = true;
                    break;
                }

                let dp_drho = (R * t / 1000.0) * (2.0 * delta * phi_d + delta.powi(2) * phi_dd);
                let dp_dt = (rho * R / 1000.0) * (delta * phi_d - delta * tau * phi_dt);
                let dh_drho = (R * t / RHO_C) * (tau * phi_dt + phi_d + delta * phi_dd);
                let dh_dt = R * (-tau.powi(2) * phi_tt + delta * phi_d - delta * tau * phi_dt);

                let det = dp_drho * dh_dt - dp_dt * dh_drho;

                if det.abs() < SOLVER_DET_TOLERANCE {
                    warn!(iter, det, "Определитель матрицы Якоби близок к нулю");
                    break;
                }

                let d_rho = (-f1 * dh_dt + f2 * dp_dt) / det;
                let d_t = (-f2 * dp_drho + f1 * dh_drho) / det;

                let mut alpha = 1.0;
                let mut next_rho = rho;
                let mut next_t = t;

                for _ in 0..5 {
                    next_rho = rho + alpha * d_rho;
                    next_t = t + alpha * d_t;
                    if next_t >= 600.0 && next_t <= 900.0 && next_rho > 1.0 && next_rho < 1500.0 { break; }
                    alpha *= 0.5;
                }
                rho = next_rho.clamp(10.0, 1500.0);
                t = next_t.clamp(620.0, 870.0);
            }
            if success { return Ok((rho, t)); }
            warn!(fallback_idx, "Неудача с текущим начальным приближением, переходим к следующему");
        }
        error!("Превышен лимит итераций во всех попытках решателя Region 3 (p, h)");
        Err(If97Error::ConvergenceError("Превышен лимит итераций в решателе Region 3 (p, h).".into()))
    }

    #[inline(always)]
    #[instrument(level = "trace")]
    fn guess_rho_t_ps(p: f64, s: f64) -> (f64, f64) {
        let p_f = ((p - P_MIN) / (P_MAX - P_MIN)) * ((P_NODES - 1) as f64);
        let s_f = ((s - S_MIN) / (S_MAX - S_MIN)) * ((S_NODES - 1) as f64);
        let p_idx = p_f.clamp(0.0, (P_NODES - 1) as f64).round() as usize;
        let s_idx = s_f.clamp(0.0, (S_NODES - 1) as f64).round() as usize;
        let guess = GRID_PS[p_idx][s_idx];
        trace!(p_idx, s_idx, ?guess, "Получено начальное приближение (rho, t) из GRID_PS");
        guess
    }

    #[instrument(level = "trace", skip(self))]
    pub fn calculate_rho_t_ps(&self, p_target: f64, s_target: f64) -> Result<(f64, f64), If97Error> {
        let (initial_rho, initial_t) = Self::guess_rho_t_ps(p_target, s_target);

        let fallback_guesses = [
            (initial_rho, initial_t),
            (initial_rho * 1.05, initial_t + 5.0),
            (initial_rho * 0.95, initial_t - 5.0),
        ];

        debug!(p_target, s_target, ?fallback_guesses, "Старт 2D решателя (p, s) в Region 3");

        for (fallback_idx, &(start_rho, start_t)) in fallback_guesses.iter().enumerate() {
            let mut rho = start_rho;
            let mut t = start_t;
            let mut success = false;

            trace!(fallback_idx, rho, t, "Попытка решения с текущим начальным приближением");

            for iter in 0..SOLVER_MAX_ITER_2D {
                let delta = rho / RHO_C;
                let tau = T_C / t;
                let phi = self.phi(delta, tau);
                let phi_d = self.phi_delta(delta, tau);
                let phi_t = self.phi_tau(delta, tau);
                let phi_dd = self.phi_delta_delta(delta, tau);
                let phi_tt = self.phi_tau_tau(delta, tau);
                let phi_dt = self.phi_delta_tau(delta, tau);

                let p_curr = rho * R * t * delta * phi_d / 1000.0;
                let s_curr = R * (tau * phi_t - phi);
                let f1 = p_curr - p_target;
                let f2 = s_curr - s_target;

                trace!(iter, p_curr, s_curr, f1, f2, "Итерация 2D решателя (p, s)");

                if f1.abs() < SOLVER_TOLERANCE && f2.abs() < SOLVER_TOLERANCE {
                    debug!(iter, rho, t, "Сходимость 2D решателя (p, s) достигнута");
                    success = true;
                    break;
                }

                let dp_drho = (R * t / 1000.0) * (2.0 * delta * phi_d + delta.powi(2) * phi_dd);
                let dp_dt = (rho * R / 1000.0) * (delta * phi_d - delta * tau * phi_dt);
                let ds_drho = (R / RHO_C) * (tau * phi_dt - phi_d);
                let ds_dt = (R / t) * (-tau.powi(2) * phi_tt);

                let det = dp_drho * ds_dt - dp_dt * ds_drho;

                if det.abs() < SOLVER_DET_TOLERANCE {
                    warn!(iter, det, "Определитель матрицы Якоби близок к нулю");
                    break;
                }

                let d_rho = (-f1 * ds_dt + f2 * dp_dt) / det;
                let d_t = (-f2 * dp_drho + f1 * ds_drho) / det;

                let mut alpha = 1.0;
                let mut next_rho = rho;
                let mut next_t = t;

                for _ in 0..5 {
                    next_rho = rho + alpha * d_rho;
                    next_t = t + alpha * d_t;
                    if next_t >= 600.0 && next_t <= 900.0 && next_rho > 1.0 && next_rho < 1500.0 { break; }
                    alpha *= 0.5;
                }
                rho = next_rho.clamp(10.0, 1500.0);
                t = next_t.clamp(620.0, 870.0);
            }
            if success { return Ok((rho, t)); }
            warn!(fallback_idx, "Неудача с текущим начальным приближением, переходим к следующему");
        }
        error!("Превышен лимит итераций во всех попытках решателя Region 3 (p, s)");
        Err(If97Error::ConvergenceError("Превышен лимит итераций в решателе Region 3 (p, s).".into()))
    }

    #[instrument(level = "debug", skip(self))]
    pub fn calculate_pt_with_guess(&self, p: f64, t: f64, rho_guess: f64) -> Result<WaterState, If97Error> {
        let rho = self.calculate_density(p, t, rho_guess);
        let v = 1.0 / rho;
        let delta = rho / RHO_C;
        let tau = T_C / t;

        let phi = self.phi(delta, tau);
        let phi_delta = self.phi_delta(delta, tau);
        let phi_tau = self.phi_tau(delta, tau);
        let phi_tau_tau = self.phi_tau_tau(delta, tau);
        let phi_delta_tau = self.phi_delta_tau(delta, tau);
        let phi_delta_delta = self.phi_delta_delta(delta, tau);

        let r_t = R * t;
        let h = r_t * (tau * phi_tau + delta * phi_delta);
        let s = R * (tau * phi_tau - phi);

        let dp_drho_term = 2.0 * delta * phi_delta + delta.powi(2) * phi_delta_delta;
        let dp_dt_term = delta * phi_delta - delta * tau * phi_delta_tau;
        let cp = R * (-tau.powi(2) * phi_tau_tau + dp_dt_term.powi(2) / dp_drho_term);

        let w_squared = r_t * 1000.0 * (dp_drho_term - dp_dt_term.powi(2) / (tau.powi(2) * phi_tau_tau));
        let w = if w_squared > 0.0 { w_squared.sqrt() } else { f64::NAN };

        debug!(v, rho, h, s, cp, w, "Успешный прямой расчет свойств Region3 (p, t)");
        Ok(WaterState {
            p: p.into(),
            t: t.into(),
            v: v.into(),
            rho: rho.into(),
            h: h.into(),
            s: s.into(),
            cp: cp.into(),
            w: w.into(),
            region: Region::Region3
        })
    }

    #[instrument(level = "debug", skip(self))]
    pub fn calculate_rhot(&self, rho: f64, t: f64) -> Result<WaterState, If97Error> {
        let v = 1.0 / rho;
        let delta = rho / RHO_C;
        let tau = T_C / t;

        let phi = self.phi(delta, tau);
        let phi_delta = self.phi_delta(delta, tau);
        let phi_tau = self.phi_tau(delta, tau);
        let phi_tau_tau = self.phi_tau_tau(delta, tau);
        let phi_delta_tau = self.phi_delta_tau(delta, tau);
        let phi_delta_delta = self.phi_delta_delta(delta, tau);

        let r_t = R * t;
        let p = rho * r_t * delta * phi_delta / 1000.0;
        let h = r_t * (tau * phi_tau + delta * phi_delta);
        let s = R * (tau * phi_tau - phi);

        let dp_drho_term = 2.0 * delta * phi_delta + delta.powi(2) * phi_delta_delta;
        let dp_dt_term = delta * phi_delta - delta * tau * phi_delta_tau;
        let cp = R * (-tau.powi(2) * phi_tau_tau + dp_dt_term.powi(2) / dp_drho_term);

        let w_squared = r_t * 1000.0 * (dp_drho_term - dp_dt_term.powi(2) / (tau.powi(2) * phi_tau_tau));
        let w = if w_squared > 0.0 { w_squared.sqrt() } else { f64::NAN };

        debug!(p, v, h, s, cp, w, "Успешный прямой расчет свойств Region3 (rho, t)");
        Ok(WaterState {
            p: p.into(),
            t: t.into(),
            v: v.into(),
            rho: rho.into(),
            h: h.into(),
            s: s.into(),
            cp: cp.into(),
            w: w.into(),
            region: Region::Region3
        })
    }
}

impl HelmholtzRegion for Region3 {
    #[instrument(level = "trace", skip(self))]
    fn phi(&self, delta: f64, tau: f64) -> f64 {
        let delta_p = Self::precompute_delta_powers(delta);
        let tau_p = Self::precompute_tau_powers(tau);
        let res = Self::N1 * delta.ln() + REGION3.iter().fold(0.0, |acc, &(n, i, j)| acc + n * delta_p[i as usize] * tau_p[j as usize]);
        trace!(res, "Рассчитана phi");
        res
    }

    #[instrument(level = "trace", skip(self))]
    fn phi_delta(&self, delta: f64, tau: f64) -> f64 {
        let delta_p = Self::precompute_delta_powers(delta);
        let tau_p = Self::precompute_tau_powers(tau);
        let res = Self::N1 / delta + REGION3.iter().fold(0.0, |acc, &(n, i, j)| {
            if i == 0 { return acc; }
            acc + n * (i as f64) * delta_p[(i - 1) as usize] * tau_p[j as usize]
        });
        trace!(res, "Рассчитана phi_delta");
        res
    }

    #[instrument(level = "trace", skip(self))]
    fn phi_delta_delta(&self, delta: f64, tau: f64) -> f64 {
        let delta_p = Self::precompute_delta_powers(delta);
        let tau_p = Self::precompute_tau_powers(tau);
        let res = -Self::N1 / (delta * delta) + REGION3.iter().fold(0.0, |acc, &(n, i, j)| {
            if i <= 1 { return acc; }
            acc + n * (i as f64) * ((i - 1) as f64) * delta_p[(i - 2) as usize] * tau_p[j as usize]
        });
        trace!(res, "Рассчитана phi_delta_delta");
        res
    }

    #[instrument(level = "trace", skip(self))]
    fn phi_tau(&self, delta: f64, tau: f64) -> f64 {
        let delta_p = Self::precompute_delta_powers(delta);
        let tau_p = Self::precompute_tau_powers(tau);
        let res = REGION3.iter().fold(0.0, |acc, &(n, i, j)| {
            if j == 0 { return acc; }
            acc + n * delta_p[i as usize] * (j as f64) * tau_p[(j - 1) as usize]
        });
        trace!(res, "Рассчитана phi_tau");
        res
    }

    #[instrument(level = "trace", skip(self))]
    fn phi_tau_tau(&self, delta: f64, tau: f64) -> f64 {
        let delta_p = Self::precompute_delta_powers(delta);
        let tau_p = Self::precompute_tau_powers(tau);
        let res = REGION3.iter().fold(0.0, |acc, &(n, i, j)| {
            if j <= 1 { return acc; }
            acc + n * delta_p[i as usize] * (j as f64) * ((j - 1) as f64) * tau_p[(j - 2) as usize]
        });
        trace!(res, "Рассчитана phi_tau_tau");
        res
    }

    #[instrument(level = "trace", skip(self))]
    fn phi_delta_tau(&self, delta: f64, tau: f64) -> f64 {
        let delta_p = Self::precompute_delta_powers(delta);
        let tau_p = Self::precompute_tau_powers(tau);
        let res = REGION3.iter().fold(0.0, |acc, &(n, i, j)| {
            if i == 0 || j == 0 { return acc; }
            acc + n * (i as f64) * delta_p[(i - 1) as usize] * (j as f64) * tau_p[(j - 1) as usize]
        });
        trace!(res, "Рассчитана phi_delta_tau");
        res
    }
}

impl WaterRegionModel for Region3 {
    #[instrument(level = "debug", skip(self))]
    fn calculate_pt(&self, p: f64, t: f64) -> Result<WaterState, If97Error> {
        debug!("Инициация расчета (p, t), начальное приближение со стороны жидкости (rho=500.0)");
        let mut rho = self.calculate_density(p, t, 500.0);

        let delta = rho / RHO_C;
        let tau = T_C / t;
        let p_calc = rho * R * t * delta * self.phi_delta(delta, tau) / 1000.0;

        if (p_calc - p).abs() > 1e-4 {
            warn!(p_calc, p_target = p, "Ошибка велика. Вероятно, застряли в жидкости. Смена начального приближения на пар (rho=150.0)");
            rho = self.calculate_density(p, t, 150.0);
        }

        self.calculate_pt_with_guess(p, t, rho)
    }

    #[instrument(level = "debug", skip(self))]
    fn calculate_ph(&self, p: f64, h: f64) -> Result<WaterState, If97Error> {
        let (rho, t) = self.calculate_rho_t_ph(p, h)?;
        if t < 623.15 || t > 863.15 {
            error!(t, "Вычисленная температура {}K вне границ Региона 3", t);
            return Err(If97Error::OutOfBounds("Точка (p, h) лежит вне температурных границ Региона 3".into()));
        }
        debug!(rho, t, "Успешно получены (rho, t) из решателя (p, h), переход к прямому расчету");
        self.calculate_pt_with_guess(p, t, rho)
    }

    #[instrument(level = "debug", skip(self))]
    fn calculate_ps(&self, p: f64, s: f64) -> Result<WaterState, If97Error> {
        let (rho, t) = self.calculate_rho_t_ps(p, s)?;
        if t < 623.15 || t > 863.15 {
            error!(t, "Вычисленная температура {}K вне границ Региона 3", t);
            return Err(If97Error::OutOfBounds("Точка (p, s) лежит вне температурных границ Региона 3".into()));
        }
        debug!(rho, t, "Успешно получены (rho, t) из решателя (p, s), переход к прямому расчету");
        self.calculate_pt_with_guess(p, t, rho)
    }
}