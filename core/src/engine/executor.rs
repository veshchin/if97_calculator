use crate::constants::{P_MAX_IF97, R, SOLVER_MAX_ITER_1D};
use crate::domain::errors::If97Error;
use crate::domain::state::{Region, WaterState};
use crate::engine::plans::{BackwardRoute, ExecutionPlan, RhoTRoute};
use crate::regions::region_1::Region1;
use crate::regions::region_2::Region2;
use crate::regions::region_2_meta::Region2Meta;
use crate::regions::region_3::Region3;
use crate::regions::region_5::Region5;
use crate::regions::traits::WaterRegionModel;
use crate::topology::boundaries::determine_region;
use crate::topology::phase_equilibrium::calculate_two_phase;

pub fn execute(plan: ExecutionPlan) -> Result<WaterState, If97Error> {
    match plan {
        ExecutionPlan::PT {
            region,
            pressure,
            temperature,
        } => execute_pt(region, pressure, temperature),
        ExecutionPlan::PH {
            route,
            pressure,
            enthalpy,
        } => execute_ph(route, pressure, enthalpy),
        ExecutionPlan::PS {
            route,
            pressure,
            entropy,
        } => execute_ps(route, pressure, entropy),
        ExecutionPlan::PX { pressure, quality } => calculate_two_phase(pressure, quality),
        ExecutionPlan::RhoT {
            route,
            density,
            temperature,
        } => execute_rhot(route, density, temperature),
        ExecutionPlan::MetastablePT {
            pressure,
            temperature,
        } => Region2Meta.calculate_pt(pressure, temperature),
    }
}

fn execute_pt(region: Region, pressure: f64, temperature: f64) -> Result<WaterState, If97Error> {
    match region {
        Region::Region1 => Region1.calculate_pt(pressure, temperature),
        Region::Region2 => Region2.calculate_pt(pressure, temperature),
        Region::Region3 => Region3.calculate_pt(pressure, temperature),
        Region::Region5 => Region5.calculate_pt(pressure, temperature),
        Region::Region4 => Err(If97Error::PhaseBoundaryError(
            "Точка лежит на линии насыщения. Используйте px(p, x).".into(),
        )),
        Region::OutOfBounds => Err(If97Error::OutOfBounds(
            "Заданные параметры (p, T) выходят за рамки стандарта IAPWS-IF97".into(),
        )),
    }
}

fn execute_ph(route: BackwardRoute, pressure: f64, enthalpy: f64) -> Result<WaterState, If97Error> {
    match route {
        BackwardRoute::Region1 => Region1.calculate_ph(pressure, enthalpy),
        BackwardRoute::Region2 => Region2.calculate_ph(pressure, enthalpy),
        BackwardRoute::Region3 => Region3.calculate_ph(pressure, enthalpy),
        BackwardRoute::Region5 => Region5.calculate_ph(pressure, enthalpy),
        BackwardRoute::TwoPhase { quality } => calculate_two_phase(pressure, quality),
    }
}

fn execute_ps(route: BackwardRoute, pressure: f64, entropy: f64) -> Result<WaterState, If97Error> {
    match route {
        BackwardRoute::Region1 => Region1.calculate_ps(pressure, entropy),
        BackwardRoute::Region2 => Region2.calculate_ps(pressure, entropy),
        BackwardRoute::Region3 => Region3.calculate_ps(pressure, entropy),
        BackwardRoute::Region5 => Region5.calculate_ps(pressure, entropy),
        BackwardRoute::TwoPhase { quality } => calculate_two_phase(pressure, quality),
    }
}

fn execute_rhot(route: RhoTRoute, density: f64, temperature: f64) -> Result<WaterState, If97Error> {
    match route {
        RhoTRoute::TwoPhase { pressure, quality } => calculate_two_phase(pressure, quality),
        RhoTRoute::NativeRegion3 => Region3.calculate_rhot(density, temperature),
        RhoTRoute::PressureSearch {
            pressure_min,
            pressure_max,
            is_supercritical,
            is_liquid_target,
        } => execute_rhot_pressure_search(
            density,
            temperature,
            pressure_min,
            pressure_max,
            is_supercritical,
            is_liquid_target,
        ),
    }
}

fn execute_rhot_pressure_search(
    density: f64,
    temperature: f64,
    pressure_min: f64,
    pressure_max: f64,
    is_supercritical: bool,
    is_liquid_target: bool,
) -> Result<WaterState, If97Error> {
    let (mut p0, mut p1) = if is_liquid_target && !is_supercritical {
        (pressure_max, pressure_max * 0.9)
    } else {
        let pressure_ideal = (density * R * temperature / 1000.0).clamp(pressure_min, pressure_max);
        let pressure_step = if pressure_ideal < pressure_max {
            (pressure_ideal * 1.01).min(pressure_max)
        } else {
            pressure_ideal * 0.99
        };
        (pressure_ideal, pressure_step)
    };

    let mut f0 = match execute_pt_for_search(p0, temperature) {
        Ok(state) => state.rho.inner() - density,
        Err(_) => {
            p0 = (p0 - 0.1).clamp(pressure_min, pressure_max);
            execute_pt_for_search(p0, temperature)?.rho.inner() - density
        }
    };

    for _ in 0..SOLVER_MAX_ITER_1D {
        let state = match execute_pt_for_search(p1, temperature) {
            Ok(state) => state,
            Err(If97Error::PhaseBoundaryError(_)) => {
                p1 = (p1 + 0.0001).clamp(pressure_min, pressure_max);
                continue;
            }
            Err(_) => {
                let step_back = (p1 - p0) / 2.0;
                if step_back.abs() < 1e-12 {
                    return execute_pt_for_search(p0, temperature);
                }
                p1 = p0 + step_back;
                continue;
            }
        };

        let current_rho = state.rho.inner();
        let is_liquid_current = current_rho > 322.0;

        if !is_supercritical && (is_liquid_target != is_liquid_current) {
            let step_back = (p1 - p0) / 2.0;
            if step_back.abs() < 1e-12 {
                return execute_pt_for_search(p0, temperature);
            }
            p1 = p0 + step_back;
            continue;
        }

        let f1 = current_rho - density;
        if f1.abs() < 1e-11 || (f1 / density).abs() < 1e-12 {
            return Ok(state);
        }

        let dp = p1 - p0;
        let df = f1 - f0;

        if df.abs() < 1e-15 {
            return Ok(state);
        }

        let step = -f1 * dp / df;
        let max_step = (pressure_max - pressure_min) * 0.8;
        let safe_step = step.clamp(-max_step, max_step);
        let p_new = (p1 + safe_step).clamp(pressure_min, pressure_max);

        if (p_new - p1).abs() < 1e-11 || (p_new - p1).abs() / p1 < 1e-10 {
            return execute_pt_for_search(p_new, temperature);
        }

        p0 = p1;
        f0 = f1;
        p1 = p_new;
    }

    Err(If97Error::ConvergenceError(String::from(
        "Convergence Error",
    )))
}

fn execute_pt_for_search(pressure: f64, temperature: f64) -> Result<WaterState, If97Error> {
    let region = determine_region(pressure, temperature);
    if pressure > P_MAX_IF97 {
        return Err(If97Error::OutOfBounds(
            "Заданные параметры (p, T) выходят за рамки стандарта IAPWS-IF97".into(),
        ));
    }
    execute_pt(region, pressure, temperature)
}
