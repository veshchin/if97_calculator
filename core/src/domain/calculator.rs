// File: src/domain/calculator.rs

use crate::domain::state::{Region, WaterState};
use crate::domain::boundaries::determine_region;
use crate::domain::traits::WaterRegionModel;
use crate::domain::errors::If97Error;
use tracing::{instrument, info, error, debug, warn, trace};

use crate::domain::models::region_1::Region1;
use crate::domain::models::region_2::Region2;
use crate::domain::models::region_2_meta::Region2Meta;
use crate::domain::models::region_3::Region3;
use crate::domain::models::region_4::{calculate_two_phase, saturation_temperature};
use crate::domain::models::region_5::Region5;

pub struct If97;

impl If97 {
    /// Основной расчет термодинамических свойств по давлению (p, МПа) и температуре (T, К)
    #[instrument(level = "info")]
    pub fn pt(p: f64, t: f64) -> Result<WaterState, If97Error> {
        let region = determine_region(p, t);
        debug!(?region, "Определен базовый регион для расчета pt");

        match region {
            Region::Region1 => Region1.calculate_pt(p, t),
            Region::Region2 => Region2.calculate_pt(p, t),
            Region::Region3 => Region3.calculate_pt(p, t),
            Region::Region5 => Region5.calculate_pt(p, t),
            Region::Region4 => {
                error!("Точка лежит на линии насыщения (p={}, t={})", p, t);
                Err(If97Error::PhaseBoundaryError("Точка лежит на линии насыщения. Используйте px(p, x).".into()))
            },
            Region::OutOfBounds => {
                error!("Заданные параметры p={}, t={} выходят за рамки стандарта IAPWS-IF97", p, t);
                Err(If97Error::OutOfBounds("Заданные параметры (p, T) выходят за рамки стандарта IAPWS-IF97".into()))
            },
        }
    }

    /// Маршрутизатор расчетов по давлению (p, МПа) и энтальпии (h, кДж/кг)
    #[instrument(level = "debug")]
    pub fn ph(p: f64, h: f64) -> Result<WaterState, If97Error> {
        if !(0.000611..=100.0).contains(&p) {
            warn!("Давление {} вне диапазона IF97", p);
            return Err(If97Error::OutOfBounds("Давление вне диапазона IF97".into()));
        }

        if p <= 22.064 {
            let t_sat = saturation_temperature(p);
            let h_liq = Region1.calculate_pt(p, t_sat)?.h;
            let h_vap = Region2.calculate_pt(p, t_sat)?.h;

            if h <= h_liq {
                debug!("Маршрутизация в Region1 (h <= h_liq)");
                return Region1.calculate_ph(p, h);
            }
            if h < h_vap {
                debug!("Маршрутизация в Region4 (двухфазная область)");
                let x = (h - h_liq) / (h_vap - h_liq);
                return calculate_two_phase(p, x);
            }
        } else {
            let h_623 = Region1.calculate_pt(p, 623.15)?.h;
            if h <= h_623 {
                debug!("Маршрутизация в Region1 (P > 22.064, h <= h_623)");
                return Region1.calculate_ph(p, h);
            }

            let t_b23 = Self::get_t_b23(p);
            let h_b23 = Region2.calculate_pt(p, t_b23)?.h;
            if h <= h_b23 {
                debug!("Маршрутизация в Region3 (h <= h_b23)");
                return Region3.calculate_ph(p, h);
            }
        }

        let h_1073 = Region2.calculate_pt(p, 1073.15)?.h;
        if h <= h_1073 {
            debug!("Маршрутизация в Region2 (h <= h_1073)");
            Region2.calculate_ph(p, h)
        } else if p <= 50.0 {
            debug!("Маршрутизация в Region5 (h > h_1073, p <= 50.0)");
            Region5.calculate_ph(p, h)
        } else {
            error!("Температура > 1073.15 K при P > 50 MPa (вне IF97)");
            Err(If97Error::OutOfBounds("Температура > 1073.15 K при P > 50 MPa (вне IF97)".into()))
        }
    }

    /// Маршрутизатор расчетов по давлению (p, МПа) и энтропии (s, кДж/(кг*К))
    #[instrument(level = "debug")]
    pub fn ps(p: f64, s: f64) -> Result<WaterState, If97Error> {
        if !(0.000611..=100.0).contains(&p) {
            warn!("Давление {} вне диапазона IF97", p);
            return Err(If97Error::OutOfBounds("Давление вне диапазона IF97".into()));
        }

        if p <= 22.064 {
            let t_sat = saturation_temperature(p);
            let s_liq = Region1.calculate_pt(p, t_sat)?.s;
            let s_vap = Region2.calculate_pt(p, t_sat)?.s;

            if s <= s_liq {
                debug!("Маршрутизация в Region1 (s <= s_liq)");
                return Region1.calculate_ps(p, s);
            }
            if s < s_vap {
                debug!("Маршрутизация в Region4 (двухфазная область)");
                let x = (s - s_liq) / (s_vap - s_liq);
                return calculate_two_phase(p, x);
            }
        } else {
            let s_623 = Region1.calculate_pt(p, 623.15)?.s;
            if s <= s_623 {
                debug!("Маршрутизация в Region1 (P > 22.064, s <= s_623)");
                return Region1.calculate_ps(p, s);
            }

            let t_b23 = Self::get_t_b23(p);
            let s_b23 = Region2.calculate_pt(p, t_b23)?.s;
            if s <= s_b23 {
                debug!("Маршрутизация в Region3 (s <= s_b23)");
                return Region3.calculate_ps(p, s);
            }
        }

        let s_1073 = Region2.calculate_pt(p, 1073.15)?.s;
        if s <= s_1073 {
            debug!("Маршрутизация в Region2 (s <= s_1073)");
            Region2.calculate_ps(p, s)
        } else if p <= 50.0 {
            debug!("Маршрутизация в Region5 (s > s_1073, p <= 50.0)");
            Region5.calculate_ps(p, s)
        } else {
            error!("Температура > 1073.15 K при P > 50 MPa (вне IF97)");
            Err(If97Error::OutOfBounds("Температура > 1073.15 K при P > 50 MPa (вне IF97)".into()))
        }
    }

    /// Расчет в двухфазной области (на линии насыщения) по давлению (p, МПа) и степени сухости (x, 0.0-1.0)
    #[instrument(level = "info")]
    pub fn px(p: f64, x: f64) -> Result<WaterState, If97Error> {
        calculate_two_phase(p, x)
    }

    /// Прямой расчет свойств по плотности (rho, кг/м3) и температуре (T, К) (только для Региона 3)
    #[instrument(level = "info")]
    pub fn rhot(rho: f64, t: f64) -> Result<WaterState, If97Error> {
        if t < 623.15 || t > 863.15 {
            error!("Температура {}K вне границ Региона 3", t);
            return Err(If97Error::OutOfBounds("Температура вне границ Региона 3 (623.15 K - 863.15 K). Для других регионов требуются итерационные решатели.".into()));
        }
        if rho <= 0.0 {
            error!("Плотность {} <= 0", rho);
            return Err(If97Error::InvalidInput("Плотность должна быть больше нуля".into()));
        }

        Region3.calculate_rhot(rho, t)
    }

    /// Расчет свойств метастабильного пара (переохлажденного пара под линией насыщения)
    #[instrument(level = "info")]
    pub fn metastable_pt(p: f64, t: f64) -> Result<WaterState, If97Error> {
        if p > 10.0 {
            error!("Метастабильный пар: давление {} MPa > 10 MPa", p);
            return Err(If97Error::OutOfBounds("Метастабильный пар по стандарту IF97 валиден только при давлении до 10 МПа".into()));
        }

        Region2Meta.calculate_pt(p, t)
    }

    #[instrument(level = "trace")]
    fn get_t_b23(p: f64) -> f64 {
        let (n3, n4, n5) = (0.10192970039326e-2, 0.57254459862746e3, 0.13918839778870e2);
        let t = n4 + ((p - n5) / n3).sqrt();
        trace!("Рассчитано значение get_t_b23: {}", t);
        t
    }
}