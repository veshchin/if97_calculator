use crate::domain::state::WaterState;
use crate::domain::errors::If97Error;

pub trait WaterRegionModel {
    fn calculate_pt(&self, p: f64, t: f64) -> Result<WaterState, If97Error>;

    fn calculate_ph(&self, _p: f64, _h: f64) -> Result<WaterState, If97Error> {
        Err(If97Error::NotImplemented("Расчет по (p, h) не реализован для данного региона".into()))
    }

    fn calculate_ps(&self, _p: f64, _s: f64) -> Result<WaterState, If97Error> {
        Err(If97Error::NotImplemented("Расчет по (p, s) не реализован для данного региона".into()))
    }
}