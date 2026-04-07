use crate::domain::errors::If97Error;
use crate::domain::state::WaterState;
use crate::domain::units::{MegaPascal, VaporFraction};
use crate::engine::{classifier, executor, validator};

pub fn run(pressure: MegaPascal, quality: VaporFraction) -> Result<WaterState, If97Error> {
    validator::validate_quality(quality.inner())?;
    let plan = classifier::classify_px(pressure.inner(), quality.inner());
    executor::execute(plan)
}
