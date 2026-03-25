use wasm_bindgen::prelude::*;
use if97_core::domain::calculator::Calculator;

// Структура, которая будет экспортирована в JavaScript
#[wasm_bindgen(getter_with_clone)]
pub struct WasmWaterState {
    pub p: f64,
    pub t: f64,
    pub v: f64,
    pub rho: f64,
    pub h: f64,
    pub s: f64,
    pub cp: f64,
    pub w: f64,
    // Передаем регион как число, так как JS не понимает Rust Enum напрямую
    pub region: i32,
}

// Функция, доступная из JavaScript
#[wasm_bindgen]
pub fn calculate_pt_js(p: f64, t: f64) -> Result<WasmWaterState, JsValue> {
    match Calculator::calculate_pt(p, t) {
        Ok(state) => {
            let region_num = match state.region {
                if97_core::domain::state::Region::Region1 => 1,
                if97_core::domain::state::Region::Region2 => 2,
                if97_core::domain::state::Region::Region3 => 3,
                if97_core::domain::state::Region::Region4 => 4,
                if97_core::domain::state::Region::Region5 => 5,
                if97_core::domain::state::Region::OutOfBounds => -1,
            };

            Ok(WasmWaterState {
                p: state.p,
                t: state.t,
                v: state.v,
                rho: state.rho,
                h: state.h,
                s: state.s,
                cp: state.cp,
                w: state.w,
                region: region_num,
            })
        },
        // Конвертируем статическую ошибку Rust в ошибку JavaScript
        Err(e) => Err(JsValue::from_str(e)),
    }
}