// src/domain/calculator.rs

use crate::domain::boundaries::determine_region;
use crate::domain::state::{Region, WaterState};
use crate::domain::traits::WaterRegionModel;

use crate::domain::models::region_1::Region1;
use crate::domain::models::region_2::Region2;
use crate::domain::models::region_2_meta::Region2Meta;
use crate::domain::models::region_3::Region3;
use crate::domain::models::region_5::Region5;

pub struct Calculator;

impl Calculator {
    /// Основной расчет термодинамических свойств по давлению (p, МПа) и температуре (T, К)
    pub fn calculate_pt(p: f64, t: f64) -> Result<WaterState, &'static str> {
        let region = determine_region(p, t);

        match region {
            Region::Region1 => Region1.calculate_pt(p, t),
            Region::Region2 => Region2.calculate_pt(p, t),
            Region::Region3 => Region3.calculate_pt(p, t),
            Region::Region5 => Region5.calculate_pt(p, t),
            Region::Region4 => Err("Точка лежит на линии насыщения. Используйте calculate_px(p, x)."),
            Region::OutOfBounds => Err("Заданные параметры (p, T) выходят за рамки стандарта IAPWS-IF97"),
        }
    }

    /// Расчет в двухфазной области (на линии насыщения) по давлению (p, МПа) и степени сухости (x, 0.0-1.0)
    pub fn calculate_px(p: f64, x: f64) -> Result<WaterState, &'static str> {
        crate::domain::models::region_4::calculate_two_phase(p, x)
    }

    /// Расчет свойств метастабильного пара (переохлажденного пара под линией насыщения)
    pub fn calc_metastable(p: f64, t: f64) -> Result<WaterState, &'static str> {
        // Уравнение IAPWS-IF97 для метастабильного пара валидно при p <= 10 MPa
        if p > 10.0 {
            return Err("Метастабильный пар по стандарту IF97 валиден только при давлении до 10 МПа");
        }

        Region2Meta.calculate_pt(p, t)
    }
}