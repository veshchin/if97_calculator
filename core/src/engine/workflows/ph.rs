use crate::domain::errors::If97Error;
use crate::domain::state::WaterState;
use crate::domain::units::{KiloJoulePerKilogram, MegaPascal};
use crate::engine::{classifier, executor, validator};

pub fn run(pressure: MegaPascal, enthalpy: KiloJoulePerKilogram) -> Result<WaterState, If97Error> {
    validator::validate_pressure(pressure.inner())?;
    let plan = classifier::classify_ph(pressure.inner(), enthalpy.inner())?;
    executor::execute(plan)
}
