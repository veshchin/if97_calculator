mod classifier;
mod executor;
mod plans;
mod request;
mod validator;
mod workflows;

use crate::domain::errors::If97Error;
use crate::domain::state::WaterState;

pub use request::CalculationRequest;

pub struct CalculationEngine;

impl CalculationEngine {
    pub fn calculate(request: CalculationRequest) -> Result<WaterState, If97Error> {
        match request {
            CalculationRequest::PT(pressure, temperature) => {
                workflows::pt::run(pressure, temperature)
            }
            CalculationRequest::PH(pressure, enthalpy) => workflows::ph::run(pressure, enthalpy),
            CalculationRequest::PS(pressure, entropy) => workflows::ps::run(pressure, entropy),
            CalculationRequest::PX(pressure, quality) => workflows::px::run(pressure, quality),
            CalculationRequest::RhoT(density, temperature) => {
                workflows::rhot::run(density, temperature)
            }
            CalculationRequest::MetastablePT(pressure, temperature) => {
                workflows::metastable_pt::run(pressure, temperature)
            }
        }
    }
}
