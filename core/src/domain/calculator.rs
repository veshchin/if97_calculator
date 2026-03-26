// src/domain/calculator.rs

use crate::domain::boundaries::determine_region;
use crate::domain::state::{Region, WaterState};
use crate::domain::traits::WaterRegionModel;

use crate::domain::models::region_1::Region1;
use crate::domain::models::region_2::Region2;
use crate::domain::models::region_2_meta::Region2Meta;
use crate::domain::models::region_3::Region3 ;
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
    /// Прямой расчет состояния по плотности (rho) и температуре (T)
    /// Доступно только для Региона 3 (околокритическая зона),
    /// так как его фундаментальное уравнение Гельмгольца базируется на (rho, T).
    /// Прямой расчет свойств по плотности (rho, кг/м3) и температуре (T, К) (только для Региона 3)
    pub fn calculate_rhot(rho: f64, t: f64) -> Result<WaterState, &'static str> {
        if t < 623.15 || t > 863.15 {
            return Err("Температура вне границ Региона 3 (623.15 K - 863.15 K). Для других регионов требуются итерационные решатели.");
        }
        if rho <= 0.0 {
            return Err("Плотность должна быть больше нуля");
        }

        // Обращаемся напрямую к математике 3-го региона
        crate::domain::models::region_3::Region3.calculate_rhot(rho, t)
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