use crate::domain::errors::If97Error;
use crate::domain::state::WaterState;
use crate::domain::units::{Kelvin, MegaPascal};
use crate::engine::{classifier, executor};

pub fn run(pressure: MegaPascal, temperature: Kelvin) -> Result<WaterState, If97Error> {
    let plan = classifier::classify_pt(pressure.inner(), temperature.inner())?;
    executor::execute(plan)
}
