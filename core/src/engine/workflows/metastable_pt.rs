use crate::domain::errors::If97Error;
use crate::domain::state::WaterState;
use crate::domain::units::{Kelvin, MegaPascal};
use crate::engine::{classifier, executor, validator};

pub fn run(pressure: MegaPascal, temperature: Kelvin) -> Result<WaterState, If97Error> {
    validator::validate_metastable_pressure(pressure.inner())?;
    let plan = classifier::classify_metastable_pt(pressure.inner(), temperature.inner());
    executor::execute(plan)
}
