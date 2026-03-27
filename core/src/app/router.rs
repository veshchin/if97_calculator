// File: src/app/router.rs

use crate::domain::state::{WaterState};
use crate::domain::models::region_1::Region1;
use crate::domain::models::region_2::Region2;
use crate::domain::models::region_3::Region3;
use crate::domain::models::region_5::Region5;
use crate::domain::models::region_4::{saturation_temperature, calculate_two_phase};
use crate::domain::traits::WaterRegionModel;
use tracing::{instrument, warn, debug, error, trace};

pub struct Router;

impl Router {
    #[instrument(level = "debug")]
    pub fn calculate_ph(p: f64, h: f64) -> Result<WaterState, &'static str> {
        if !(0.000611..=100.0).contains(&p) {
            warn!("Давление {} вне диапазона IF97", p);
            return Err("Давление вне диапазона IF97");
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
                // Вычисляем степень сухости по энтальпии и пробрасываем в Регион 4
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
            Err("Температура > 1073.15 K при P > 50 MPa (вне IF97)")
        }
    }

    #[instrument(level = "debug")]
    pub fn calculate_ps(p: f64, s: f64) -> Result<WaterState, &'static str> {
        if !(0.000611..=100.0).contains(&p) {
            warn!("Давление {} вне диапазона IF97", p);
            return Err("Давление вне диапазона IF97");
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
                // Вычисляем степень сухости по энтропии и пробрасываем в Регион 4
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
            Err("Температура > 1073.15 K при P > 50 MPa (вне IF97)")
        }
    }

    #[instrument(level = "trace")]
    fn get_t_b23(p: f64) -> f64 {
        let (n3, n4, n5) = (0.10192970039326e-2, 0.57254459862746e3, 0.13918839778870e2);
        let t = n4 + ((p - n5) / n3).sqrt();
        trace!("Рассчитано значение get_t_b23: {}", t);
        t
    }
}