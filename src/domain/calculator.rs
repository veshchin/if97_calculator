use crate::boundaries::determine_region;
use crate::constants::{R, RHO_C, T_C};
use crate::math::ThermodynamicRegion;
use crate::state::{Region, WaterState};

use crate::backward::{region1_t_ph, region1_t_ps};
use crate::region_1::Region1;

use crate::region_2_meta::Region2Meta;
use crate::backward::{region2_t_ph, region2_t_ps};
use crate::region_2::Region2;
use crate::region_3::Region3;
use crate::region_4::saturation_temperature;
use crate::region_5::Region5;

use crate::backward::{region5_t_ph, region5_t_ps};

pub struct Calculator;

impl Calculator {
    pub fn calculate(p: f64, t: f64) -> Result<WaterState, &'static str> {
        let region = determine_region(p, t);
        match region {
            Region::Region1 => Ok(Self::calc_gibbs(p, t, 16.53, 1386.0, &Region1, region)),
            Region::Region2 => Ok(Self::calc_gibbs(p, t, 1.0, 540.0, &Region2, region)),
            Region::Region5 => Ok(Self::calc_gibbs(p, t, 1.0, 1000.0, &Region5, region)),
            Region::Region3 => Ok(Self::calc_helmholtz(p, t, &Region3)),
            Region::Region4 => Err("Точка лежит на линии насыщения. Используйте calculate_two_phase(p, x)."),
            Region::OutOfBounds => Err("Заданные параметры (p, T) выходят за рамки стандарта IAPWS-IF97"),
        }
    }

    #[allow(dead_code)]
    pub fn calculate_two_phase(p: f64, x: f64) -> Result<WaterState, &'static str> {
        if !(0.0..=1.0).contains(&x) {
            return Err("Степень сухости x должна быть в диапазоне от 0.0 до 1.0");
        }
        let t_sat = saturation_temperature(p);
        let state_liquid = Self::calc_gibbs(p, t_sat, 16.53, 1386.0, &Region1, Region::Region4);
        let state_vapor = Self::calc_gibbs(p, t_sat, 1.0, 540.0, &Region2, Region::Region4);

        let v = state_liquid.v + x * (state_vapor.v - state_liquid.v);
        let h = state_liquid.h + x * (state_vapor.h - state_liquid.h);
        let s = state_liquid.s + x * (state_vapor.s - state_liquid.s);

        Ok(WaterState { p, t: t_sat, v, rho: 1.0 / v, h, s, cp: f64::NAN, w: f64::NAN, region: Region::Region4 })
    }

    fn calc_gibbs<T: ThermodynamicRegion>(
        p: f64, t: f64, p_star: f64, t_star: f64, region_math: &T, region_enum: Region
    ) -> WaterState {
        let pi = p / p_star;
        let tau = t_star / t;

        let gamma = region_math.gamma(pi, tau);
        let gamma_pi = region_math.gamma_pi(pi, tau);
        let gamma_tau = region_math.gamma_tau(pi, tau);
        let gamma_pi_pi = region_math.gamma_pi_pi(pi, tau);
        let gamma_tau_tau = region_math.gamma_tau_tau(pi, tau);
        let gamma_pi_tau = region_math.gamma_pi_tau(pi, tau);

        let v = 0.001 * (R * t / p) * pi * gamma_pi;
        let h = R * t * tau * gamma_tau;
        let s = R * (tau * gamma_tau - gamma);
        let cp = -R * tau.powi(2) * gamma_tau_tau;

        // Строгий аналитический вывод скорости звука без неоднозначности знаков
        let w_squared = (R * t * gamma_pi.powi(2)) /
            ( (gamma_pi - tau * gamma_pi_tau).powi(2) / (tau.powi(2) * gamma_tau_tau) - gamma_pi_pi );

        let w = if w_squared > 0.0 { (w_squared * 1000.0).sqrt() } else { f64::NAN };

        WaterState { p, t, v, rho: 1.0 / v, h, s, cp, w, region: region_enum }
    }

    // src/calculator.rs

    fn calc_helmholtz(p: f64, t: f64, region3: &Region3) -> WaterState {
        // Сначала решатель находит плотность
        let rho = region3.calculate_density(p, t);
        let v = 1.0 / rho;

        // Безразмерные параметры для 3 региона (базируются на критических значениях)
        let delta = rho / RHO_C;
        let tau = T_C / t;

        // Вычисляем саму функцию и ее производные
        let phi = region3.phi(delta, tau);
        let phi_delta = region3.phi_delta(delta, tau);
        let phi_tau = region3.phi_tau(delta, tau);
        let phi_tau_tau = region3.phi_tau_tau(delta, tau);
        let phi_delta_tau = region3.phi_delta_tau(delta, tau);
        let phi_delta_delta = region3.phi_delta_delta(delta, tau);

        let r_t = R * t;

        // Энтальпия и энтропия
        let h = r_t * (tau * phi_tau + delta * phi_delta);
        let s = R * (tau * phi_tau - phi);

        // Вспомогательные термы для теплоемкости и скорости звука
        let dp_drho_term = 2.0 * delta * phi_delta + delta.powi(2) * phi_delta_delta;
        let dp_dt_term = delta * phi_delta - delta * tau * phi_delta_tau;

        // Изобарная теплоемкость (cp)
        let cp = R * (-tau.powi(2) * phi_tau_tau + dp_dt_term.powi(2) / dp_drho_term);

        // Скорость звука (w)
        let w_squared = r_t * 1000.0 * (dp_drho_term - dp_dt_term.powi(2) / (tau.powi(2) * phi_tau_tau));
        let w = if w_squared > 0.0 { w_squared.sqrt() } else { f64::NAN };

        WaterState { p, t, v, rho, h, s, cp, w, region: Region::Region3 }
    }

    /// Сверхбыстрый расчет свойств Региона 1 по давлению (p) и энтальпии (h)
    pub fn calc_region1_ph(p: f64, h: f64) -> Result<WaterState, &'static str> {
        // 1. Находим температуру через обратное уравнение (O(1) сложность, без итераций)
        let t = region1_t_ph(p, h);

        // 2. Опциональная проверка границ (защита от "мусорных" входных данных)
        if t < 273.15 || t > 623.15 || p > 100.0 {
            return Err("Точка (p, h) лежит вне границ Региона 1");
        }

        // 3. Вызываем вашу уже готовую и отлаженную функцию calc_gibbs
        Ok(Self::calc_gibbs(p, t, 16.53, 1386.0, &Region1, Region::Region1))
    }

    /// Сверхбыстрый расчет свойств Региона 1 по давлению (p) и энтропии (s)
    pub fn calc_region1_ps(p: f64, s: f64) -> Result<WaterState, &'static str> {
        // 1. Мгновенно вычисляем температуру
        let t = region1_t_ps(p, s);

        // 2. Проверка границ
        if t < 273.15 || t > 623.15 || p > 100.0 {
            return Err("Точка (p, s) лежит вне границ Региона 1");
        }

        // 3. Вызываем прямое вычисление
        Ok(Self::calc_gibbs(p, t, 16.53, 1386.0, &Region1, Region::Region1))
    }

    pub fn calc_region2_ph(p: f64, h: f64) -> Result<WaterState, &'static str> {
        let t = region2_t_ph(p, h);

        // Защита от выхода за границы Региона 2
        if t < 273.15 || t > 1073.15 || p > 100.0 {
            return Err("Точка (p, h) лежит вне границ Региона 2");
        }

        Ok(Self::calc_gibbs(p, t, 1.0, 540.0, &Region2, Region::Region2))
    }

    pub fn calc_region2_ps(p: f64, s: f64) -> Result<WaterState, &'static str> {
        let t = region2_t_ps(p, s);

        if t < 273.15 || t > 1073.15 || p > 100.0 {
            return Err("Точка (p, s) лежит вне границ Региона 2");
        }

        Ok(Self::calc_gibbs(p, t, 1.0, 540.0, &Region2, Region::Region2))
    }
    /// Расчет свойств метастабильного пара (переохлажденного пара под линией насыщения)
    pub fn calc_metastable(p: f64, t: f64) -> Result<WaterState, &'static str> {
        // Уравнение IAPWS-IF97 для метастабильного пара валидно при p <= 10 MPa
        if p > 10.0 {
            return Err("Метастабильный пар по стандарту IF97 валиден только при давлении до 10 МПа");
        }

        // Обратите внимание: возвращаем Region::Region2, так как метастабильная зона
        // является расширением второго региона (или вы можете добавить Metstable в enum Region).
        Ok(Self::calc_gibbs(p, t, 1.0, 540.0, &Region2Meta, Region::Region2))
    }

    // Добавьте это внутри impl Calculator в src/calculator.rs

    /// Обратный расчет свойств Региона 3 по давлению (p) и энтальпии (h)
    pub fn calc_region3_ph(p: f64, h: f64) -> Result<WaterState, &'static str> {
        let region3 = Region3;

        // 1. Вызываем наш 2D решатель Ньютона-Рафсона для нахождения rho и T
        let (_rho, t) = region3.calculate_rho_t_ph(p, h)?;

        // 2. Проверяем границы температур для Региона 3 (от 623.15 K до 863.15 K)
        if t < 623.15 || t > 863.15 {
            return Err("Найденная точка (p, h) лежит вне температурных границ Региона 3");
        }

        // 3. Формируем полное состояние, используя уже отлаженный метод Гельмгольца
        Ok(Self::calc_helmholtz(p, t, &region3))
    }

    /// Обратный расчет свойств Региона 3 по давлению (p) и энтропии (s)
    pub fn calc_region3_ps(p: f64, s: f64) -> Result<WaterState, &'static str> {
        let region3 = Region3;

        // 1. Находим плотность и температуру итерационно
        let (_rho, t) = region3.calculate_rho_t_ps(p, s)?;

        // 2. Проверяем границы
        if t < 623.15 || t > 863.15 {
            return Err("Найденная точка (p, s) лежит вне температурных границ Региона 3");
        }

        // 3. Возвращаем полное термодинамическое состояние
        Ok(Self::calc_helmholtz(p, t, &region3))
    }
    /// Сверхбыстрый расчет свойств Региона 5 по давлению (p) и энтальпии (h)
    pub fn calc_region5_ph(p: f64, h: f64) -> Result<WaterState, &'static str> {
        // 1. Итерационно находим температуру (сойдется за 3-5 итераций)
        let t = region5_t_ph(p, h)?;

        // 2. Жесткая проверка границ Региона 5 по стандарту IAPWS-IF97
        if t < 1073.15 || t > 2273.15 || p < 0.0 || p > 50.0 {
            return Err("Точка (p, h) лежит вне границ Региона 5");
        }

        // 3. Вызываем расчет полного состояния (p_star = 1.0, t_star = 1000.0)
        Ok(Self::calc_gibbs(p, t, 1.0, 1000.0, &Region5, Region::Region5))
    }

    /// Сверхбыстрый расчет свойств Региона 5 по давлению (p) и энтропии (s)
    pub fn calc_region5_ps(p: f64, s: f64) -> Result<WaterState, &'static str> {
        // 1. Итерационно находим температуру
        let t = region5_t_ps(p, s)?;

        // 2. Проверка физических границ
        if t < 1073.15 || t > 2273.15 || p < 0.0 || p > 50.0 {
            return Err("Точка (p, s) лежит вне границ Региона 5");
        }

        // 3. Возвращаем полное термодинамическое состояние
        Ok(Self::calc_gibbs(p, t, 1.0, 1000.0, &Region5, Region::Region5))
    }
}