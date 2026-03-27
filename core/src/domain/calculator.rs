// File: src/domain/calculator.rs

use crate::domain::state::{Region, WaterState};
use crate::domain::boundaries::determine_region;
use crate::domain::traits::WaterRegionModel;
use crate::domain::errors::If97Error;
use crate::domain::units::*;
use tracing::{instrument, info, error, debug, warn, trace};

use crate::domain::models::region_1::Region1;
use crate::domain::models::region_2::Region2;
use crate::domain::models::region_2_meta::Region2Meta;
use crate::domain::models::region_3::Region3;
use crate::domain::models::region_4::{calculate_two_phase, saturation_temperature};
use crate::domain::models::region_5::Region5;
use crate::domain::constants::*;

pub struct If97;

impl If97 {
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

    #[instrument(level = "debug")]
    pub fn ph(p: MegaPascal, h: KiloJoulePerKilogram) -> Result<WaterState, If97Error> {
        let p_val = p.inner();
        let h_val = h.inner();

        if !(P_MIN_IF97..=P_MAX_IF97).contains(&p_val) {
            warn!("Давление {} вне диапазона IF97", p_val);
            return Err(If97Error::OutOfBounds("Давление вне диапазона IF97".into()));
        }

        if p_val <= P_C {
            let t_sat = saturation_temperature(p_val);
            let h_liq = Region1.calculate_pt(p_val, t_sat)?.h.inner();
            let h_vap = Region2.calculate_pt(p_val, t_sat)?.h.inner();

            if h_val <= h_liq {
                debug!("Маршрутизация в Region1 (h <= h_liq)");
                return Region1.calculate_ph(p_val, h_val);
            }
            if h_val < h_vap {
                debug!("Маршрутизация в Region4 (двухфазная область)");
                let x_val = (h_val - h_liq) / (h_vap - h_liq);
                return calculate_two_phase(p_val, x_val);
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

    #[instrument(level = "debug")]
    pub fn ps(p: MegaPascal, s: KiloJoulePerKilogramKelvin) -> Result<WaterState, If97Error> {
        let p_val = p.inner();
        let s_val = s.inner();

        if !(P_MIN_IF97..=P_MAX_IF97).contains(&p_val) {
            warn!("Давление {} вне диапазона IF97", p_val);
            return Err(If97Error::OutOfBounds("Давление вне диапазона IF97".into()));
        }

        if p_val <= P_C {
            let t_sat = saturation_temperature(p_val);
            let s_liq = Region1.calculate_pt(p_val, t_sat)?.s.inner();
            let s_vap = Region2.calculate_pt(p_val, t_sat)?.s.inner();

            if s_val <= s_liq {
                debug!("Маршрутизация в Region1 (s <= s_liq)");
                return Region1.calculate_ps(p_val, s_val);
            }
            if s_val < s_vap {
                debug!("Маршрутизация в Region4 (двухфазная область)");
                let x_val = (s_val - s_liq) / (s_vap - s_liq);
                return calculate_two_phase(p_val, x_val);
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

    #[instrument(level = "info")]
    pub fn px(p: MegaPascal, x: VaporFraction) -> Result<WaterState, If97Error> {
        calculate_two_phase(p.inner(), x.inner())
    }

    #[instrument(level = "info")]
    pub fn rhot(rho: KilogramPerCubicMeter, t: Kelvin) -> Result<WaterState, If97Error> {
        let rho_val = rho.inner();
        let t_val = t.inner();

        if t_val < 623.15 || t_val > 863.15 {
            error!("Температура {}K вне границ Региона 3", t_val);
            return Err(If97Error::OutOfBounds("Температура вне границ Региона 3 (623.15 K - 863.15 K). Для других регионов требуются итерационные решатели.".into()));
        }
        if rho_val <= 0.0 {
            error!("Плотность {} <= 0", rho_val);
            return Err(If97Error::InvalidInput("Плотность должна быть больше нуля".into()));
        }

        Region3.calculate_rhot(rho_val, t_val)
    }

    #[instrument(level = "info")]
    pub fn metastable_pt(p: MegaPascal, t: Kelvin) -> Result<WaterState, If97Error> {
        let p_val = p.inner();
        let t_val = t.inner();

        if p_val > 10.0 {
            error!("Метастабильный пар: давление {} MPa > 10 MPa", p_val);
            return Err(If97Error::OutOfBounds("Метастабильный пар по стандарту IF97 валиден только при давлении до 10 МПа".into()));
        }

        Region2Meta.calculate_pt(p_val, t_val)
    }

    #[instrument(level = "trace")]
    fn get_t_b23(p: f64) -> f64 {
        let (n3, n4, n5) = (0.10192970039326e-2, 0.57254459862746e3, 0.13918839778870e2);
        let t = n4 + ((p - n5) / n3).sqrt();
        trace!("Рассчитано значение get_t_b23: {}", t);
        t
    }
}