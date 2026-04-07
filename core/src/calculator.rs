// File: src/calculator.rs

use crate::state::{Region, WaterState};
use crate::models::boundaries::determine_region;
use crate::models::traits::WaterRegionModel;
use crate::errors::If97Error;
use crate::units::*;
use tracing::{instrument, error, debug, warn, trace};

use crate::models::region_1::Region1;
use crate::models::region_2::Region2;
use crate::models::region_2_meta::Region2Meta;
use crate::models::region_3::Region3;
use crate::models::region_4::{calculate_two_phase, saturation_pressure};
use crate::models::region_5::Region5;
use crate::constants::*;

/// Варианты входных параметров для универсальной точки входа `calculate`.
pub enum InputParams {
    /// Абсолютное давление (МПа) и температура (К).
    PT(MegaPascal, Kelvin),
    /// Плотность (кг/м³) и температура (К).
    RhoT(KilogramPerCubicMeter, Kelvin),
}

/// Главный вычислительный фасад библиотеки.
///
/// Предоставляет публичные методы для расчета теплофизических свойств воды и пара
/// по различным входным парам параметров (p-T, p-h, p-s, p-x, rho-T).
/// Автоматически маршрутизирует запрос в нужный регион стандарта IAPWS-IF97.
pub struct If97;

impl If97 {
    /// Универсальная точка входа для расчета свойств воды и пара.
    ///
    /// Позволяет передавать параметры через перечисление `InputParams`,
    /// что удобно для создания обобщенных интерфейсов поверх библиотеки.
    #[instrument(level = "info", skip(input))]
    pub fn calculate(input: InputParams) -> Result<WaterState, If97Error> {
        match input {
            InputParams::PT(p, t) => {
                debug!("Вызван универсальный метод calculate с параметрами PT");
                Self::pt(p, t)
            },
            InputParams::RhoT(rho, t) => {
                debug!("Вызван универсальный метод calculate с параметрами RhoT");
                Self::rhot(rho, t)
            },
        }
    }

    /// Расчет свойств по заданному давлению и температуре.
    ///
    /// # Arguments
    /// * `p` - Абсолютное давление в мегапаскалях (МПа).
    /// * `t` - Температура в кельвинах (К).
    ///
    /// # Returns
    /// Возвращает структуру `WaterState` со всеми рассчитанными теплофизическими свойствами.
    ///
    /// # Errors
    /// * `If97Error::OutOfBounds` - если параметры выходят за границы стандарта IAPWS-IF97.
    /// * `If97Error::PhaseBoundaryError` - если точка лежит точно на линии насыщения (Region 4). Для таких точек следует использовать метод `px`.
    #[instrument(level = "info")]
    pub fn pt(p: MegaPascal, t: Kelvin) -> Result<WaterState, If97Error> {
        let p_val = p.inner();
        let t_val = t.inner();
        let region = determine_region(p_val, t_val);
        debug!(?region, "Определен базовый регион для расчета pt");

        match region {
            Region::Region1 => Region1.calculate_pt(p_val, t_val),
            Region::Region2 => Region2.calculate_pt(p_val, t_val),
            Region::Region3 => Region3.calculate_pt(p_val, t_val),
            Region::Region5 => Region5.calculate_pt(p_val, t_val),
            Region::Region4 => {
                error!("Точка лежит на линии насыщения (p={}, t={})", p_val, t_val);
                Err(If97Error::PhaseBoundaryError("Точка лежит на линии насыщения. Используйте px(p, x).".into()))
            },
            Region::OutOfBounds => {
                error!("Заданные параметры p={}, t={} выходят за рамки стандарта IAPWS-IF97", p_val, t_val);
                Err(If97Error::OutOfBounds("Заданные параметры (p, T) выходят за рамки стандарта IAPWS-IF97".into()))
            },
        }
    }

    /// Обратный расчет свойств по заданному давлению и удельной энтальпии.
    ///
    /// Метод определяет текущее фазовое состояние (жидкость, двухфазная смесь или пар) путем
    /// сравнения заданной энтальпии с энтальпиями на границах насыщения, после чего
    /// вызывает решатель соответствующего региона.
    ///
    /// # Arguments
    /// * `p` - Абсолютное давление в мегапаскалях (МПа).
    /// * `h` - Удельная энтальпия в кДж/кг.
    ///
    /// # Returns
    /// Возвращает структуру `WaterState`.
    ///
    /// # Errors
    /// Возвращает `If97Error::OutOfBounds`, если точка вне стандарта, или ошибку сходимости, если итерационный решатель не нашел решение.
    #[instrument(level = "debug")]
    pub fn ph(p: MegaPascal, h: KiloJoulePerKilogram) -> Result<WaterState, If97Error> {
        let p_val = p.inner();
        let h_val = h.inner();

        if !(P_MIN_IF97..=P_MAX_IF97).contains(&p_val) {
            warn!("Давление {} вне диапазона IF97", p_val);
            return Err(If97Error::OutOfBounds("Давление вне диапазона IF97".into()));
        }

        if p_val <= P_C {
            // Корректное определение границ насыщения через Region 4 (учитывает Region 3)
            let state_liq = calculate_two_phase(p_val, 0.0)?;
            let state_vap = calculate_two_phase(p_val, 1.0)?;
            let h_liq = state_liq.h.inner();
            let h_vap = state_vap.h.inner();
            let t_sat = state_liq.t.inner();

            if h_val <= h_liq {
                if t_sat <= 623.15 {
                    debug!("Маршрутизация в Region1 (h <= h_liq)");
                    return Region1.calculate_ph(p_val, h_val);
                } else {
                    debug!("Маршрутизация в Region3 (жидкость, h <= h_liq)");
                    return Region3.calculate_ph(p_val, h_val);
                }
            }
            if h_val < h_vap {
                debug!("Маршрутизация в Region4 (двухфазная область)");
                let x_val = (h_val - h_liq) / (h_vap - h_liq);
                return calculate_two_phase(p_val, x_val);
            }

            if t_sat > 623.15 {
                let t_b23 = Self::get_t_b23(p_val);
                let h_b23 = Region2.calculate_pt(p_val, t_b23)?.h.inner();
                if h_val <= h_b23 {
                    debug!("Маршрутизация в Region3 (пар, h <= h_b23)");
                    return Region3.calculate_ph(p_val, h_val);
                }
            }
        } else {
            let h_623 = Region1.calculate_pt(p_val, 623.15)?.h.inner();
            if h_val <= h_623 {
                debug!("Маршрутизация в Region1 (P > 22.064, h <= h_623)");
                return Region1.calculate_ph(p_val, h_val);
            }

            let t_b23 = Self::get_t_b23(p_val);
            let h_b23 = Region2.calculate_pt(p_val, t_b23)?.h.inner();
            if h_val <= h_b23 {
                debug!("Маршрутизация в Region3 (h <= h_b23)");
                return Region3.calculate_ph(p_val, h_val);
            }
        }

        let h_1073 = Region2.calculate_pt(p_val, T_MAX_REGION1_2)?.h.inner();
        if h_val <= h_1073 {
            debug!("Маршрутизация в Region2 (h <= h_1073)");
            Region2.calculate_ph(p_val, h_val)
        } else if p_val <= P_MAX_REGION5 {
            debug!("Маршрутизация в Region5 (h > h_1073, p <= 50.0)");
            Region5.calculate_ph(p_val, h_val)
        } else {
            error!("Температура > 1073.15 K при P > 50 MPa (вне IF97)");
            Err(If97Error::OutOfBounds("Температура > 1073.15 K при P > 50 MPa (вне IF97)".into()))
        }
    }

    /// Обратный расчет свойств по заданному давлению и удельной энтропии.
    ///
    /// Работает аналогично методу `ph`, маршрутизируя запрос на основе сравнения
    /// текущей энтропии со значениями энтропии на граничных линиях стандарта.
    ///
    /// # Arguments
    /// * `p` - Абсолютное давление в мегапаскалях (МПа).
    /// * `s` - Удельная энтропия в кДж/(кг·К).
    ///
    /// # Returns
    /// Возвращает структуру `WaterState`.
    ///
    /// # Errors
    /// Возвращает ошибки выхода за границы или ошибки сходимости итерационных решателей.
    #[instrument(level = "debug")]
    pub fn ps(p: MegaPascal, s: KiloJoulePerKilogramKelvin) -> Result<WaterState, If97Error> {
        let p_val = p.inner();
        let s_val = s.inner();

        if !(P_MIN_IF97..=P_MAX_IF97).contains(&p_val) {
            warn!("Давление {} вне диапазона IF97", p_val);
            return Err(If97Error::OutOfBounds("Давление вне диапазона IF97".into()));
        }

        if p_val <= P_C {
            // Корректное определение границ насыщения через Region 4 (учитывает Region 3)
            let state_liq = calculate_two_phase(p_val, 0.0)?;
            let state_vap = calculate_two_phase(p_val, 1.0)?;
            let s_liq = state_liq.s.inner();
            let s_vap = state_vap.s.inner();
            let t_sat = state_liq.t.inner();

            if s_val <= s_liq {
                if t_sat <= 623.15 {
                    debug!("Маршрутизация в Region1 (s <= s_liq)");
                    return Region1.calculate_ps(p_val, s_val);
                } else {
                    debug!("Маршрутизация в Region3 (жидкость, s <= s_liq)");
                    return Region3.calculate_ps(p_val, s_val);
                }
            }
            if s_val < s_vap {
                debug!("Маршрутизация в Region4 (двухфазная область)");
                let x_val = (s_val - s_liq) / (s_vap - s_liq);
                return calculate_two_phase(p_val, x_val);
            }

            if t_sat > 623.15 {
                let t_b23 = Self::get_t_b23(p_val);
                let s_b23 = Region2.calculate_pt(p_val, t_b23)?.s.inner();
                if s_val <= s_b23 {
                    debug!("Маршрутизация в Region3 (пар, s <= s_b23)");
                    return Region3.calculate_ps(p_val, s_val);
                }
            }
        } else {
            let s_623 = Region1.calculate_pt(p_val, 623.15)?.s.inner();
            if s_val <= s_623 {
                debug!("Маршрутизация в Region1 (P > 22.064, s <= s_623)");
                return Region1.calculate_ps(p_val, s_val);
            }

            let t_b23 = Self::get_t_b23(p_val);
            let s_b23 = Region2.calculate_pt(p_val, t_b23)?.s.inner();
            if s_val <= s_b23 {
                debug!("Маршрутизация в Region3 (s <= s_b23)");
                return Region3.calculate_ps(p_val, s_val);
            }
        }

        let s_1073 = Region2.calculate_pt(p_val, T_MAX_REGION1_2)?.s.inner();
        if s_val <= s_1073 {
            debug!("Маршрутизация в Region2 (s <= s_1073)");
            Region2.calculate_ps(p_val, s_val)
        } else if p_val <= P_MAX_REGION5 {
            debug!("Маршрутизация в Region5 (s > s_1073, p <= 50.0)");
            Region5.calculate_ps(p_val, s_val)
        } else {
            error!("Температура > 1073.15 K при P > 50 MPa (вне IF97)");
            Err(If97Error::OutOfBounds("Температура > 1073.15 K при P > 50 MPa (вне IF97)".into()))
        }
    }

    /// Расчет свойств влажного пара (двухфазная область, Region 4).
    ///
    /// # Arguments
    /// * `p` - Абсолютное давление в мегапаскалях (МПа).
    /// * `x` - Степень сухости пара (массовая доля пара в смеси) от 0.0 до 1.0.
    ///
    /// # Returns
    /// Свойства двухфазной смеси, усредненные по правилу аддитивности.
    /// Скорость звука и теплоемкость для влажного пара не рассчитываются (возвращается NaN).
    #[instrument(level = "info")]
    pub fn px(p: MegaPascal, x: VaporFraction) -> Result<WaterState, If97Error> {
        debug!(p = p.inner(), x = x.inner(), "Запрос расчета двухфазной области px");
        calculate_two_phase(p.inner(), x.inner())
    }

    /// Расчет свойств по заданной плотности и температуре.
    ///
    /// Использует метод секущих для нахождения давления, соответствующего целевой плотности.
    /// Является универсальным входом для задач гидродинамики, где плотность известна изначально.
    ///
    /// # Arguments
    /// * `rho` - Плотность в кг/м³.
    /// * `t` - Температура в кельвинах (К).
    #[instrument(level = "info")]
    pub fn rhot(rho: KilogramPerCubicMeter, t: Kelvin) -> Result<WaterState, If97Error> {
        let rho_val = rho.inner();
        let t_val = t.inner();

        if rho_val <= 0.0 {
            error!("Недопустимая плотность: {} <= 0", rho_val);
            return Err(If97Error::InvalidInput(String::from("Invalid input")));
        }

        // Защита от передачи мусорных температур в формулы насыщения
        if t_val < 273.15 || t_val > 2273.15 {
            return Err(If97Error::OutOfBounds(String::from("Temperature out of bounds")));
        }

        let is_supercritical = t_val > 647.096;

        if (273.15..=647.096).contains(&t_val) {
            let p_sat = saturation_pressure(t_val);
            if let Ok(liq) = calculate_two_phase(p_sat, 0.0) {
                if let Ok(vap) = calculate_two_phase(p_sat, 1.0) {
                    let rho_liq = liq.rho.inner();
                    let rho_vap = vap.rho.inner();

                    if rho_val <= rho_liq + 1e-10 && rho_val >= rho_vap - 1e-10 {
                        let v_val = 1.0 / rho_val;
                        let v_liq = liq.v.inner();
                        let v_vap = vap.v.inner();

                        let x = (v_val - v_liq) / (v_vap - v_liq);
                        let x_clamped = x.clamp(0.0, 1.0);

                        return calculate_two_phase(p_sat, x_clamped);
                    }
                }
            }
        }

        if (623.15..=863.15).contains(&t_val) {
            if let Ok(state) = Region3.calculate_rhot(rho_val, t_val) {
                if determine_region(state.p.inner(), t_val) == Region::Region3 {
                    debug!("Успешный расчет rhot через нативный Region3");
                    return Ok(state);
                }
            }
        }

        debug!(rho = rho_val, t = t_val, "Запуск 1D решателя (метод секущих) для rhot");
        let mut p_max_for_t = if t_val > 1073.15 { 50.0 } else { P_MAX_IF97 };
        let mut p_min_safe = P_MIN_IF97.max(0.000611657) + 1e-9;

        let is_liquid_target = rho_val > 322.0;

        if !is_supercritical {
            let p_sat = saturation_pressure(t_val);
            if is_liquid_target {
                p_min_safe = p_min_safe.max(p_sat + 1e-8);
            } else {
                p_max_for_t = p_max_for_t.min(p_sat - 1e-8);
            }
        }

        // ВАЖНО: Предотвращаем Panic в clamp() если диапазон сузился в минус из-за невозможных параметров
        if p_max_for_t <= p_min_safe {
            return Err(If97Error::OutOfBounds(String::from("No valid pressure range")));
        }

        let (mut p0, mut p1) = if is_liquid_target && !is_supercritical {
            (p_max_for_t, p_max_for_t * 0.9)
        } else {
            let p_ideal = (rho_val * R * t_val / 1000.0).clamp(p_min_safe, p_max_for_t);
            let p_step = if p_ideal < p_max_for_t { (p_ideal * 1.01).min(p_max_for_t) } else { p_ideal * 0.99 };
            (p_ideal, p_step)
        };

        let mut f0 = match Self::pt(MegaPascal(p0), t) {
            Ok(state) => state.rho.inner() - rho_val,
            Err(_) => {
                p0 = (p0 - 0.1).clamp(p_min_safe, p_max_for_t);
                Self::pt(MegaPascal(p0), t)?.rho.inner() - rho_val
            }
        };

        for iter in 0..SOLVER_MAX_ITER_1D {
            let state = match Self::pt(MegaPascal(p1), t) {
                Ok(s) => s,
                Err(If97Error::PhaseBoundaryError(_)) => {
                    p1 = (p1 + 0.0001).clamp(p_min_safe, p_max_for_t);
                    continue;
                },
                Err(_) => {
                    let step_back = (p1 - p0) / 2.0;
                    if step_back.abs() < 1e-12 {
                        return Self::pt(MegaPascal(p0), t);
                    }
                    p1 = p0 + step_back;
                    continue;
                }
            };

            let current_rho = state.rho.inner();
            let is_liquid_current = current_rho > 322.0;

            if !is_supercritical && (is_liquid_target != is_liquid_current) {
                let step_back = (p1 - p0) / 2.0;
                if step_back.abs() < 1e-12 {
                    return Self::pt(MegaPascal(p0), t);
                }
                p1 = p0 + step_back;
                continue;
            }

            let f1 = current_rho - rho_val;
            if f1.abs() < 1e-11 || (f1 / rho_val).abs() < 1e-12 {
                debug!(iter, p_final = p1, "Сходимость решателя rhot достигнута по плотности");
                return Ok(state);
            }

            let dp = p1 - p0;
            let df = f1 - f0;

            if df.abs() < 1e-15 {
                return Ok(state);
            }

            let step = -f1 * dp / df;
            let max_step = (p_max_for_t - p_min_safe) * 0.8;

            // Теперь max_step гарантированно > 0, и clamp не убьет WASM-поток
            let safe_step = step.clamp(-max_step, max_step);

            let p_new = (p1 + safe_step).clamp(p_min_safe, p_max_for_t);
            if (p_new - p1).abs() < 1e-11 || (p_new - p1).abs() / p1 < 1e-10 {
                debug!(iter, p_final = p_new, "Сходимость решателя rhot достигнута по давлению");
                return Self::pt(MegaPascal(p_new), t);
            }

            p0 = p1;
            f0 = f1;
            p1 = p_new;
        }

        error!("Решатель rhot не смог достичь сходимости за {} итераций", SOLVER_MAX_ITER_1D);
        Err(If97Error::ConvergenceError(String::from("Convergence Error")))
    }

    /// Расчет свойств метастабильного переохлажденного пара (расширение Region 2).
    ///
    /// Применяется для состояний пара, охлажденного ниже температуры насыщения без конденсации
    /// (например, в паровых турбинах при быстром расширении).
    ///
    /// # Arguments
    /// * `p` - Абсолютное давление в мегапаскалях (МПа).
    /// * `t` - Температура в кельвинах (К).
    ///
    /// # Errors
    /// Возвращает ошибку, если давление превышает 10 МПа (ограничение стандарта для метастабильной зоны).
    #[instrument(level = "info")]
    pub fn metastable_pt(p: MegaPascal, t: Kelvin) -> Result<WaterState, If97Error> {
        let p_val = p.inner();
        let t_val = t.inner();

        if p_val > 10.0 {
            error!("Метастабильный пар: давление {} MPa > 10 MPa", p_val);
            return Err(If97Error::OutOfBounds("Метастабильный пар по стандарту IF97 валиден только при давлении до 10 МПа".into()));
        }

        debug!(p = p_val, t = t_val, "Запрос расчета свойств метастабильного пара");
        Region2Meta.calculate_pt(p_val, t_val)
    }

    /// Расчет температуры на границе B23 (между Регионами 2 и 3) по заданному давлению.
    #[instrument(level = "trace")]
    fn get_t_b23(p: f64) -> f64 {
        let (n3, n4, n5) = (0.10192970039326e-2, 0.57254459862746e3, 0.13918839778870e2);
        let t = n4 + ((p - n5) / n3).sqrt();
        trace!("Рассчитано значение get_t_b23: {}", t);
        t
    }
}