use crate::constants::{P_C, P_MAX_IF97, P_MAX_REGION5, P_MIN_IF97, R, T_MAX_REGION1_2};
use crate::domain::errors::If97Error;
use crate::domain::state::Region;
use crate::engine::plans::{BackwardRoute, ExecutionPlan, RhoTRoute};
use crate::regions::region_1::Region1;
use crate::regions::region_2::Region2;
use crate::regions::region_3::Region3;
use crate::regions::traits::WaterRegionModel;
use crate::topology::boundaries::{b23_temperature, determine_region};
use crate::topology::phase_equilibrium::{calculate_two_phase, saturation_pressure};

pub fn classify_pt(pressure: f64, temperature: f64) -> Result<ExecutionPlan, If97Error> {
    let region = determine_region(pressure, temperature);
    match region {
        Region::Region1 | Region::Region2 | Region::Region3 | Region::Region5 => {
            Ok(ExecutionPlan::PT {
                region,
                pressure,
                temperature,
            })
        }
        Region::Region4 => Err(If97Error::PhaseBoundaryError(
            "Точка лежит на линии насыщения. Используйте px(p, x).".into(),
        )),
        Region::OutOfBounds => Err(If97Error::OutOfBounds(
            "Заданные параметры (p, T) выходят за рамки стандарта IAPWS-IF97".into(),
        )),
    }
}

pub fn classify_ph(pressure: f64, enthalpy: f64) -> Result<ExecutionPlan, If97Error> {
    if !(P_MIN_IF97..=P_MAX_IF97).contains(&pressure) {
        return Err(If97Error::OutOfBounds("Давление вне диапазона IF97".into()));
    }

    if pressure <= P_C {
        let state_liq = calculate_two_phase(pressure, 0.0)?;
        let state_vap = calculate_two_phase(pressure, 1.0)?;
        let h_liq = state_liq.h.inner();
        let h_vap = state_vap.h.inner();
        let t_sat = state_liq.t.inner();

        if enthalpy <= h_liq {
            let route = if t_sat <= 623.15 {
                BackwardRoute::Region1
            } else {
                BackwardRoute::Region3
            };
            return Ok(ExecutionPlan::PH {
                route,
                pressure,
                enthalpy,
            });
        }

        if enthalpy < h_vap {
            return Ok(ExecutionPlan::PH {
                route: BackwardRoute::TwoPhase {
                    quality: (enthalpy - h_liq) / (h_vap - h_liq),
                },
                pressure,
                enthalpy,
            });
        }

        if t_sat > 623.15 {
            let t_b23 = b23_temperature(pressure);
            let h_b23 = Region2.calculate_pt(pressure, t_b23)?.h.inner();
            if enthalpy <= h_b23 {
                return Ok(ExecutionPlan::PH {
                    route: BackwardRoute::Region3,
                    pressure,
                    enthalpy,
                });
            }
        }
    } else {
        let h_623 = Region1.calculate_pt(pressure, 623.15)?.h.inner();
        if enthalpy <= h_623 {
            return Ok(ExecutionPlan::PH {
                route: BackwardRoute::Region1,
                pressure,
                enthalpy,
            });
        }

        let t_b23 = b23_temperature(pressure);
        let h_b23 = Region2.calculate_pt(pressure, t_b23)?.h.inner();
        if enthalpy <= h_b23 {
            return Ok(ExecutionPlan::PH {
                route: BackwardRoute::Region3,
                pressure,
                enthalpy,
            });
        }
    }

    let h_1073 = Region2.calculate_pt(pressure, T_MAX_REGION1_2)?.h.inner();
    if enthalpy <= h_1073 {
        Ok(ExecutionPlan::PH {
            route: BackwardRoute::Region2,
            pressure,
            enthalpy,
        })
    } else if pressure <= P_MAX_REGION5 {
        Ok(ExecutionPlan::PH {
            route: BackwardRoute::Region5,
            pressure,
            enthalpy,
        })
    } else {
        Err(If97Error::OutOfBounds(
            "Температура > 1073.15 K при P > 50 MPa (вне IF97)".into(),
        ))
    }
}

pub fn classify_ps(pressure: f64, entropy: f64) -> Result<ExecutionPlan, If97Error> {
    if !(P_MIN_IF97..=P_MAX_IF97).contains(&pressure) {
        return Err(If97Error::OutOfBounds("Давление вне диапазона IF97".into()));
    }

    if pressure <= P_C {
        let state_liq = calculate_two_phase(pressure, 0.0)?;
        let state_vap = calculate_two_phase(pressure, 1.0)?;
        let s_liq = state_liq.s.inner();
        let s_vap = state_vap.s.inner();
        let t_sat = state_liq.t.inner();

        if entropy <= s_liq {
            let route = if t_sat <= 623.15 {
                BackwardRoute::Region1
            } else {
                BackwardRoute::Region3
            };
            return Ok(ExecutionPlan::PS {
                route,
                pressure,
                entropy,
            });
        }

        if entropy < s_vap {
            return Ok(ExecutionPlan::PS {
                route: BackwardRoute::TwoPhase {
                    quality: (entropy - s_liq) / (s_vap - s_liq),
                },
                pressure,
                entropy,
            });
        }

        if t_sat > 623.15 {
            let t_b23 = b23_temperature(pressure);
            let s_b23 = Region2.calculate_pt(pressure, t_b23)?.s.inner();
            if entropy <= s_b23 {
                return Ok(ExecutionPlan::PS {
                    route: BackwardRoute::Region3,
                    pressure,
                    entropy,
                });
            }
        }
    } else {
        let s_623 = Region1.calculate_pt(pressure, 623.15)?.s.inner();
        if entropy <= s_623 {
            return Ok(ExecutionPlan::PS {
                route: BackwardRoute::Region1,
                pressure,
                entropy,
            });
        }

        let t_b23 = b23_temperature(pressure);
        let s_b23 = Region2.calculate_pt(pressure, t_b23)?.s.inner();
        if entropy <= s_b23 {
            return Ok(ExecutionPlan::PS {
                route: BackwardRoute::Region3,
                pressure,
                entropy,
            });
        }
    }

    let s_1073 = Region2.calculate_pt(pressure, T_MAX_REGION1_2)?.s.inner();
    if entropy <= s_1073 {
        Ok(ExecutionPlan::PS {
            route: BackwardRoute::Region2,
            pressure,
            entropy,
        })
    } else if pressure <= P_MAX_REGION5 {
        Ok(ExecutionPlan::PS {
            route: BackwardRoute::Region5,
            pressure,
            entropy,
        })
    } else {
        Err(If97Error::OutOfBounds(
            "Температура > 1073.15 K при P > 50 MPa (вне IF97)".into(),
        ))
    }
}

pub fn classify_px(pressure: f64, quality: f64) -> ExecutionPlan {
    ExecutionPlan::PX { pressure, quality }
}

pub fn classify_rhot(density: f64, temperature: f64) -> Result<ExecutionPlan, If97Error> {
    if (273.15..=647.096).contains(&temperature) {
        let pressure_sat = saturation_pressure(temperature);
        if let Ok(liq) = calculate_two_phase(pressure_sat, 0.0) {
            if let Ok(vap) = calculate_two_phase(pressure_sat, 1.0) {
                let rho_liq = liq.rho.inner();
                let rho_vap = vap.rho.inner();

                if density <= rho_liq + 1e-10 && density >= rho_vap - 1e-10 {
                    let volume = 1.0 / density;
                    let volume_liq = liq.v.inner();
                    let volume_vap = vap.v.inner();
                    let quality =
                        ((volume - volume_liq) / (volume_vap - volume_liq)).clamp(0.0, 1.0);

                    return Ok(ExecutionPlan::RhoT {
                        route: RhoTRoute::TwoPhase {
                            pressure: pressure_sat,
                            quality,
                        },
                        density,
                        temperature,
                    });
                }
            }
        }
    }

    if (623.15..=863.15).contains(&temperature) {
        if let Ok(state) = Region3.calculate_rhot(density, temperature) {
            if determine_region(state.p.inner(), temperature) == Region::Region3 {
                return Ok(ExecutionPlan::RhoT {
                    route: RhoTRoute::NativeRegion3,
                    density,
                    temperature,
                });
            }
        }
    }

    let is_supercritical = temperature > 647.096;
    let is_liquid_target = density > 322.0;
    let mut pressure_max = if temperature > 1073.15 {
        50.0
    } else {
        P_MAX_IF97
    };
    let mut pressure_min = P_MIN_IF97.max(0.000611657) + 1e-9;

    if !is_supercritical {
        let pressure_sat = saturation_pressure(temperature);
        if is_liquid_target {
            pressure_min = pressure_min.max(pressure_sat + 1e-8);
        } else {
            pressure_max = pressure_max.min(pressure_sat - 1e-8);
        }
    }

    if pressure_max <= pressure_min {
        return Err(If97Error::OutOfBounds(String::from(
            "No valid pressure range",
        )));
    }

    let _pressure_ideal = (density * R * temperature / 1000.0).clamp(pressure_min, pressure_max);
    Ok(ExecutionPlan::RhoT {
        route: RhoTRoute::PressureSearch {
            pressure_min,
            pressure_max,
            is_supercritical,
            is_liquid_target,
        },
        density,
        temperature,
    })
}

pub fn classify_metastable_pt(pressure: f64, temperature: f64) -> ExecutionPlan {
    ExecutionPlan::MetastablePT {
        pressure,
        temperature,
    }
}
