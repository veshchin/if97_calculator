use crate::constants::{R, T_C, RHO_C};
use crate::tables::REGION3;

// Константы для сетки начальных приближений Региона 3
const P_MIN: f64 = 16.0;
const P_MAX: f64 = 100.0;
const P_NODES: usize = 4;

const H_MIN: f64 = 1600.0;
const H_MAX: f64 = 3800.0;
const H_NODES: usize = 4;

const S_MIN: f64 = 3.8;
const S_MAX: f64 = 6.4;
const S_NODES: usize = 4;

// Двумерная сетка стартовых приближений (rho, T) для пары (p, h)
// Структура: GRID_PH[индекс_давления][индекс_энтальпии]
const GRID_PH: [[(f64, f64); 4]; 4] = [
    [(850.0, 625.0), (600.0, 635.0), (200.0, 650.0), ( 20.0, 750.0)], // p ≈ 16 MPa
    [(870.0, 635.0), (650.0, 680.0), (350.0, 710.0), ( 40.0, 800.0)], // p ≈ 44 MPa
    [(900.0, 645.0), (700.0, 710.0), (480.0, 760.0), ( 80.0, 840.0)], // p ≈ 72 MPa
    [(920.0, 650.0), (730.0, 730.0), (550.0, 780.0), (100.0, 860.0)], // p ≈ 100 MPa
];

// Двумерная сетка стартовых приближений (rho, T) для пары (p, s)
const GRID_PS: [[(f64, f64); 4]; 4] = [
    [(850.0, 625.0), (600.0, 635.0), (200.0, 650.0), ( 20.0, 750.0)], // p ≈ 16 MPa
    [(870.0, 635.0), (650.0, 680.0), (350.0, 710.0), ( 40.0, 800.0)], // p ≈ 44 MPa
    [(900.0, 645.0), (700.0, 710.0), (480.0, 760.0), ( 80.0, 840.0)], // p ≈ 72 MPa
    [(920.0, 650.0), (730.0, 730.0), (550.0, 780.0), (100.0, 860.0)], // p ≈ 100 MPa
];

pub struct Region3;

impl Region3 {
    #[inline(always)]
    fn precompute_delta_powers(delta: f64) -> [f64; 15] {
        let mut powers = [0.0; 15];
        powers[0] = 1.0;
        for i in 1..15 { powers[i] = powers[i - 1] * delta; }
        powers
    }

    #[inline(always)]
    fn precompute_tau_powers(tau: f64) -> [f64; 40] {
        let mut powers = [0.0; 40];
        powers[0] = 1.0;
        for j in 1..40 { powers[j] = powers[j - 1] * tau; }
        powers
    }

    // Коэффициент n_1 по Таблице 30 стандарта IAPWS-IF97
    const N1: f64 = 1.0658070028513;

    pub fn phi(&self, delta: f64, tau: f64) -> f64 {
        let delta_powers = Self::precompute_delta_powers(delta);
        let tau_powers = Self::precompute_tau_powers(tau);

        let mut res = Self::N1 * delta.ln();
        res += REGION3.iter().fold(0.0, |acc, &(n, i, j)| {
            acc + n * delta_powers[i as usize] * tau_powers[j as usize]
        });
        res
    }

    pub fn phi_delta(&self, delta: f64, tau: f64) -> f64 {
        let delta_powers = Self::precompute_delta_powers(delta);
        let tau_powers = Self::precompute_tau_powers(tau);

        let mut res = Self::N1 / delta;
        res += REGION3.iter().fold(0.0, |acc, &(n, i, j)| {
            if i == 0 { return acc; }
            acc + n * (i as f64) * delta_powers[(i - 1) as usize] * tau_powers[j as usize]
        });
        res
    }

    pub fn phi_delta_delta(&self, delta: f64, tau: f64) -> f64 {
        let delta_powers = Self::precompute_delta_powers(delta);
        let tau_powers = Self::precompute_tau_powers(tau);

        let mut res = -Self::N1 / (delta * delta);
        res += REGION3.iter().fold(0.0, |acc, &(n, i, j)| {
            if i <= 1 { return acc; }
            acc + n * (i as f64) * ((i - 1) as f64) * delta_powers[(i - 2) as usize] * tau_powers[j as usize]
        });
        res
    }

    pub fn phi_tau(&self, delta: f64, tau: f64) -> f64 {
        let delta_powers = Self::precompute_delta_powers(delta);
        let tau_powers = Self::precompute_tau_powers(tau);

        REGION3.iter().fold(0.0, |acc, &(n, i, j)| {
            if j == 0 { return acc; }
            acc + n * delta_powers[i as usize] * (j as f64) * tau_powers[(j - 1) as usize]
        })
    }

    pub fn phi_tau_tau(&self, delta: f64, tau: f64) -> f64 {
        let delta_powers = Self::precompute_delta_powers(delta);
        let tau_powers = Self::precompute_tau_powers(tau);

        REGION3.iter().fold(0.0, |acc, &(n, i, j)| {
            if j <= 1 { return acc; }
            acc + n * delta_powers[i as usize] * (j as f64) * ((j - 1) as f64) * tau_powers[(j - 2) as usize]
        })
    }

    pub fn phi_delta_tau(&self, delta: f64, tau: f64) -> f64 {
        let delta_powers = Self::precompute_delta_powers(delta);
        let tau_powers = Self::precompute_tau_powers(tau);

        REGION3.iter().fold(0.0, |acc, &(n, i, j)| {
            if i == 0 || j == 0 { return acc; }
            acc + n * (i as f64) * delta_powers[(i - 1) as usize] * (j as f64) * tau_powers[(j - 1) as usize]
        })
    }

    // Итерационный поиск плотности (rho) по заданным давлению (p) и температуре (t)
    pub fn calculate_density(&self, p: f64, t: f64) -> f64 {
        let tau = T_C / t;
        let mut rho_guess = 500.0;
        let mut delta = rho_guess / RHO_C;

        let tolerance = 1e-8;
        let max_iters = 100;

        for _ in 0..max_iters {
            let pd = self.phi_delta(delta, tau);

            // p = rho * R * T * delta * phi_delta (делим на 1000 для MPa)
            let p_calc = rho_guess * R * t * delta * pd / 1000.0;
            let error = p_calc - p;

            if error.abs() < tolerance { break; }

            let pdd = self.phi_delta_delta(delta, tau);

            // Производная давления по плотности dp/drho для метода Ньютона
            let dp_drho = R * t * (2.0 * delta * pd + delta.powi(2) * pdd) / 1000.0;

            let mut step = error / dp_drho;

            // Ограничение шага (не более 10% от текущего значения), чтобы алгоритм не "улетел"
            if step > 0.1 * rho_guess { step = 0.1 * rho_guess; }
            if step < -0.1 * rho_guess { step = -0.1 * rho_guess; }

            rho_guess -= step;
            delta = rho_guess / RHO_C;
        }
        rho_guess
    }
    /// 2D Newton-Raphson решатель для (p, h) -> (rho, T) с демпфированием и клиппированием
    /// O(1) поиск ближайшего стартового приближения (rho_0, T_0) по (p, h)
    #[inline(always)]
    fn guess_rho_t_ph(p: f64, h: f64) -> (f64, f64) {
        let p_idx = (((p - P_MIN) / (P_MAX - P_MIN)) * ((P_NODES - 1) as f64)).round() as usize;
        let h_idx = (((h - H_MIN) / (H_MAX - H_MIN)) * ((H_NODES - 1) as f64)).round() as usize;
        GRID_PH[p_idx.clamp(0, P_NODES - 1)][h_idx.clamp(0, H_NODES - 1)]
    }

    /// O(1) поиск ближайшего стартового приближения (rho_0, T_0) по (p, s)
    #[inline(always)]
    fn guess_rho_t_ps(p: f64, s: f64) -> (f64, f64) {
        let p_idx = (((p - P_MIN) / (P_MAX - P_MIN)) * ((P_NODES - 1) as f64)).round() as usize;
        let s_idx = (((s - S_MIN) / (S_MAX - S_MIN)) * ((S_NODES - 1) as f64)).round() as usize;
        GRID_PS[p_idx.clamp(0, P_NODES - 1)][s_idx.clamp(0, S_NODES - 1)]
    }
    /// Low Latency 2D Newton-Raphson решатель для (p, h) -> (rho, T)
    /// Low Latency 2D Newton-Raphson решатель для (p, h) -> (rho, T)
    pub fn calculate_rho_t_ph(&self, p_target: f64, h_target: f64) -> Result<(f64, f64), &'static str> {
        // Получаем идеальное стартовое приближение за O(1) из нашей сетки GRID_PH
        let (mut rho, mut t) = Self::guess_rho_t_ph(p_target, h_target);

        let tol = 1e-7;
        let max_iters = 50; // Оставляем запас

        for _ in 0..max_iters {
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

            if f1.abs() < tol && f2.abs() < tol {
                return Ok((rho, t));
            }

            let dp_drho = (R * t / 1000.0) * (2.0 * delta * phi_d + delta.powi(2) * phi_dd);
            let dp_dt = (rho * R / 1000.0) * (delta * phi_d - delta * tau * phi_dt);
            let dh_drho = (R * t / RHO_C) * (tau * phi_dt + phi_d + delta * phi_dd);

            // ИСПРАВЛЕННАЯ ПРОИЗВОДНАЯ: убрано лишнее слагаемое tau * phi_t
            let dh_dt = R * (-tau.powi(2) * phi_tt + delta * phi_d - delta * tau * phi_dt);

            let det = dp_drho * dh_dt - dp_dt * dh_drho;
            if det.abs() < 1e-12 {
                return Err("Сингулярный Якобиан (деление на ноль) при расчете Region 3 (p, h)");
            }

            let d_rho = (-f1 * dh_dt + f2 * dp_dt) / det;
            let d_t = (-f2 * dp_drho + f1 * dh_drho) / det;

            // Демпфирование шага (Line Search) для гашения выбросов
            let mut alpha = 1.0;
            let mut next_rho = rho;
            let mut next_t = t;

            for _ in 0..5 {
                next_rho = rho + alpha * d_rho;
                next_t = t + alpha * d_t;

                // Физические границы Region 3 с запасом
                if next_t >= 600.0 && next_t <= 900.0 && next_rho > 1.0 && next_rho < 1500.0 {
                    break;
                }
                alpha *= 0.5;
            }

            // Жесткое клиппирование переменных
            rho = next_rho.clamp(10.0, 1500.0);
            t = next_t.clamp(620.0, 870.0);
        }

        Err("Превышен лимит итераций в решателе Region 3 (p, h).")
    }

    /// Low Latency 2D Newton-Raphson решатель для (p, s) -> (rho, T)
    pub fn calculate_rho_t_ps(&self, p_target: f64, s_target: f64) -> Result<(f64, f64), &'static str> {
        // Получаем идеальное стартовое приближение за O(1)
        let (mut rho, mut t) = Self::guess_rho_t_ps(p_target, s_target);

        let tol = 1e-7;
        let max_iters = 50;

        for _ in 0..max_iters {
            let delta = rho / RHO_C;
            let tau = T_C / t;

            let phi_val = self.phi(delta, tau);
            let phi_d = self.phi_delta(delta, tau);
            let phi_t = self.phi_tau(delta, tau);
            let phi_dd = self.phi_delta_delta(delta, tau);
            let phi_tt = self.phi_tau_tau(delta, tau);
            let phi_dt = self.phi_delta_tau(delta, tau);

            let p_curr = rho * R * t * delta * phi_d / 1000.0;
            let s_curr = R * (tau * phi_t - phi_val);

            let f1 = p_curr - p_target;
            let f2 = s_curr - s_target;

            if f1.abs() < tol && f2.abs() < tol {
                return Ok((rho, t));
            }

            let dp_drho = (R * t / 1000.0) * (2.0 * delta * phi_d + delta.powi(2) * phi_dd);
            let dp_dt = (rho * R / 1000.0) * (delta * phi_d - delta * tau * phi_dt);
            let ds_drho = (R / RHO_C) * (tau * phi_dt - phi_d);
            let ds_dt = -(R * tau.powi(2) / t) * phi_tt;

            let det = dp_drho * ds_dt - dp_dt * ds_drho;
            if det.abs() < 1e-12 {
                return Err("Сингулярный Якобиан (деление на ноль) при расчете Region 3 (p, s)");
            }

            let d_rho = (-f1 * ds_dt + f2 * dp_dt) / det;
            let d_t = (-f2 * dp_drho + f1 * ds_drho) / det;

            let mut alpha = 1.0;
            let mut next_rho = rho;
            let mut next_t = t;

            for _ in 0..5 {
                next_rho = rho + alpha * d_rho;
                next_t = t + alpha * d_t;

                if next_t >= 600.0 && next_t <= 900.0 && next_rho > 1.0 && next_rho < 1500.0 {
                    break;
                }
                alpha *= 0.5;
            }

            rho = next_rho.clamp(10.0, 1500.0);
            t = next_t.clamp(620.0, 870.0);
        }

        Err("Превышен лимит итераций в решателе Region 3 (p, s).")
    }
}
