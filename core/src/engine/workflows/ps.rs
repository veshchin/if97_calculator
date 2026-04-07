use crate::domain::errors::If97Error;
use crate::domain::state::WaterState;
use crate::domain::units::{KiloJoulePerKilogramKelvin, MegaPascal};
use crate::engine::{classifier, executor, validator};

pub fn run(
    pressure: MegaPascal,
    entropy: KiloJoulePerKilogramKelvin,
) -> Result<WaterState, If97Error> {
    validator::validate_pressure(pressure.inner())?;
    let plan = classifier::classify_ps(pressure.inner(), entropy.inner())?;
    executor::execute(plan)
}
