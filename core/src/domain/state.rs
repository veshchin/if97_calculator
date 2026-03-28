use serde::{Serialize, Deserialize};
use crate::domain::units::*;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Region {
    #[default]
    OutOfBounds,
    Region1, Region2, Region3, Region4, Region5,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct WaterState {
    pub p: MegaPascal,
    pub t: Kelvin,
    pub v: CubicMeterPerKilogram,
    pub rho: KilogramPerCubicMeter,
    pub h: KiloJoulePerKilogram,
    pub s: KiloJoulePerKilogramKelvin,
    pub cp: KiloJoulePerKilogramKelvin,
    pub w: MeterPerSecond,
    pub region: Region,
    pub u: KiloJoulePerKilogram,
}