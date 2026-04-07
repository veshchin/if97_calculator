use crate::domain::state::Region;

#[derive(Debug, Clone, Copy)]
pub enum BackwardRoute {
    Region1,
    Region2,
    Region3,
    Region5,
    TwoPhase { quality: f64 },
}

#[derive(Debug, Clone, Copy)]
pub enum RhoTRoute {
    TwoPhase {
        pressure: f64,
        quality: f64,
    },
    NativeRegion3,
    PressureSearch {
        pressure_min: f64,
        pressure_max: f64,
        is_supercritical: bool,
        is_liquid_target: bool,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum ExecutionPlan {
    PT {
        region: Region,
        pressure: f64,
        temperature: f64,
    },
    PH {
        route: BackwardRoute,
        pressure: f64,
        enthalpy: f64,
    },
    PS {
        route: BackwardRoute,
        pressure: f64,
        entropy: f64,
    },
    PX {
        pressure: f64,
        quality: f64,
    },
    RhoT {
        route: RhoTRoute,
        density: f64,
        temperature: f64,
    },
    MetastablePT {
        pressure: f64,
        temperature: f64,
    },
}
