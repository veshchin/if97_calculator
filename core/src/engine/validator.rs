use crate::constants::{P_MAX_IF97, P_MIN_IF97, T_MAX_REGION5, T_MIN_IF97};
use crate::domain::errors::If97Error;

pub fn validate_pressure(pressure: f64) -> Result<(), If97Error> {
    if !(P_MIN_IF97..=P_MAX_IF97).contains(&pressure) {
        return Err(If97Error::OutOfBounds("Давление вне диапазона IF97".into()));
    }
    Ok(())
}

pub fn validate_density(density: f64) -> Result<(), If97Error> {
    if density <= 0.0 {
        return Err(If97Error::InvalidInput(String::from("Invalid input")));
    }
    Ok(())
}

pub fn validate_temperature(temperature: f64) -> Result<(), If97Error> {
    if !(T_MIN_IF97..=T_MAX_REGION5).contains(&temperature) {
        return Err(If97Error::OutOfBounds(String::from(
            "Temperature out of bounds",
        )));
    }
    Ok(())
}

pub fn validate_quality(quality: f64) -> Result<(), If97Error> {
    if !(0.0..=1.0).contains(&quality) {
        return Err(If97Error::InvalidInput(
            "Степень сухости x должна быть в диапазоне от 0.0 до 1.0".into(),
        ));
    }
    Ok(())
}

pub fn validate_metastable_pressure(pressure: f64) -> Result<(), If97Error> {
    if pressure > 10.0 {
        return Err(If97Error::OutOfBounds(
            "Метастабильный пар по стандарту IF97 валиден только при давлении до 10 МПа".into(),
        ));
    }
    Ok(())
}
