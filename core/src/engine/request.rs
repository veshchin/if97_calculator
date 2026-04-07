use crate::domain::units::*;

#[derive(Debug, Clone, Copy)]
pub enum CalculationRequest {
    PT(MegaPascal, Kelvin),
    PH(MegaPascal, KiloJoulePerKilogram),
    PS(MegaPascal, KiloJoulePerKilogramKelvin),
    PX(MegaPascal, VaporFraction),
    RhoT(KilogramPerCubicMeter, Kelvin),
    MetastablePT(MegaPascal, Kelvin),
}
