use crate::domain::state::WaterState;

pub trait WaterRegionModel {
    fn calculate_pt(&self, p: f64, t: f64) -> Result<WaterState, &'static str>;

    fn calculate_ph(&self, _p: f64, _h: f64) -> Result<WaterState, &'static str> {
        Err("Расчет по (p, h) не реализован для данного региона")
    }

    fn calculate_ps(&self, _p: f64, _s: f64) -> Result<WaterState, &'static str> {
        Err("Расчет по (p, s) не реализован для данного региона")
    }
}