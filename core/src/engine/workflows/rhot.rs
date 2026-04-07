use crate::domain::errors::If97Error;
use crate::domain::state::WaterState;
use crate::domain::units::{Kelvin, KilogramPerCubicMeter};
use crate::engine::{classifier, executor, validator};

pub fn run(density: KilogramPerCubicMeter, temperature: Kelvin) -> Result<WaterState, If97Error> {
    validator::validate_density(density.inner())?;
    validator::validate_temperature(temperature.inner())?;
    let plan = classifier::classify_rhot(density.inner(), temperature.inner())?;
    executor::execute(plan)
}
