use crate::tauri_api;
use crate::types::{AppContext, SavedItem, SavedPoint, StateContext};
use if97_app_api::{InputMode, SingleCalcRequest, StateDto};
use std::cell::RefCell;
use std::rc::Rc;
use web_sys::{HtmlInputElement, HtmlSelectElement, MouseEvent};
use yew::prelude::*;

fn parse_input_value(value: &str) -> Result<f64, String> {
    value
        .trim()
        .replace(',', ".")
        .parse::<f64>()
        .map_err(|_| "Ошибка чтения".to_string())
}

fn format_value(value: f64, precision: usize, unit: &str) -> String {
    let numeric = if value.is_infinite() {
        if value.is_sign_negative() {
            "-∞".to_string()
        } else {
            "∞".to_string()
        }
    } else if value.is_nan() {
        "NaN".to_string()
    } else {
        format!("{:.*}", precision, value)
    };
    format!("{numeric} {unit}")
}

fn matching_request(mode: InputMode, v1: &str, v2: &str, point: &SavedPoint) -> bool {
    if point.orig_mode != mode {
        return false;
    }

    match (
        parse_input_value(v1),
        parse_input_value(v2),
        parse_input_value(&point.orig_v1),
        parse_input_value(&point.orig_v2),
    ) {
        (Ok(left_v1), Ok(left_v2), Ok(right_v1), Ok(right_v2)) => {
            left_v1 == right_v1 && left_v2 == right_v2
        }
        _ => false,
    }
}

fn labels_for_mode(mode: InputMode) -> (&'static str, &'static str) {
    match mode {
        InputMode::Pt => ("Давление p (МПа)", "Температура T (К)"),
        InputMode::Rhot => ("Плотность rho (кг/м^3)", "Температура T (К)"),
        InputMode::Px => ("Давление p (МПа)", "Степень сухости x"),
        InputMode::Ps => ("Давление p (МПа)", "Энтропия s (кДж/кгК)"),
        InputMode::Ph => ("Давление p (МПа)", "Энтальпия h (кДж/кг)"),
    }
}

fn update_name_from_saved(
    app_ctx: &crate::types::SavedItemsState,
    custom_name: &UseStateHandle<String>,
    mode: InputMode,
    v1: &str,
    v2: &str,
) {
    if let Some(existing) = app_ctx.items.iter().find_map(|item| match item {
        SavedItem::Point(point) if matching_request(mode, v1, v2, point) => Some(point),
        _ => None,
    }) {
        custom_name.set(existing.name.clone());
    }
}

fn is_saved_request(
    app_ctx: &crate::types::SavedItemsState,
    mode: InputMode,
    v1: &str,
    v2: &str,
) -> bool {
    app_ctx.items.iter().any(|item| match item {
        SavedItem::Point(point) => matching_request(mode, v1, v2, point),
        SavedItem::Table(_) => false,
    })
}

fn run_single_calculation(
    snapshot: crate::types::PersistentState,
    state_ctx: StateContext,
    app_ctx: AppContext,
    custom_name: UseStateHandle<String>,
    request_seq: Rc<RefCell<u64>>,
) {
    if snapshot.s_v1.trim().is_empty() || snapshot.s_v2.trim().is_empty() {
        let mut next = snapshot.clone();
        next.s_res = None;
        next.s_error = None;
        state_ctx.set(next);
        return;
    }

    let v1 = match parse_input_value(&snapshot.s_v1) {
        Ok(value) => value,
        Err(error) => {
            let mut next = snapshot.clone();
            next.s_res = None;
            next.s_error = Some(error);
            state_ctx.set(next);
            return;
        }
    };

    let v2 = match parse_input_value(&snapshot.s_v2) {
        Ok(value) => value,
        Err(error) => {
            let mut next = snapshot.clone();
            next.s_res = None;
            next.s_error = Some(error);
            state_ctx.set(next);
            return;
        }
    };

    let ticket = {
        let mut seq = request_seq.borrow_mut();
        *seq += 1;
        *seq
    };

    wasm_bindgen_futures::spawn_local(async move {
        let result = tauri_api::calculate_single(SingleCalcRequest {
            mode: snapshot.s_mode,
            v1,
            v2,
        })
        .await;

        if *request_seq.borrow() != ticket {
            return;
        }

        let mut next = (*state_ctx).clone();
        if next.s_mode != snapshot.s_mode || next.s_v1 != snapshot.s_v1 || next.s_v2 != snapshot.s_v2
        {
            return;
        }

        match result {
            Ok(state) => {
                next.s_res = Some(state);
                next.s_error = None;
                update_name_from_saved(&app_ctx, &custom_name, next.s_mode, &next.s_v1, &next.s_v2);
            }
            Err(error) => {
                next.s_res = None;
                next.s_error = Some(error);
            }
        }

        state_ctx.set(next);
    });
}

fn view_result(result: &StateDto, precision: usize) -> Html {
    html! {
        <div class="card fade-in">
            <h3>{"Результат расчета"}</h3>
            <div class="results-grid">
                <div class="result-item"><span class="label">{"Давление (p)"}</span><span class="val">{format_value(result.p, precision, "МПа")}</span></div>
                <div class="result-item"><span class="label">{"Температура (T)"}</span><span class="val">{format_value(result.t, precision, "К")}</span></div>
                <div class="result-item"><span class="label">{"Удельный объем (v)"}</span><span class="val">{format_value(result.v, precision, "м^3/кг")}</span></div>
                <div class="result-item"><span class="label">{"Плотность (rho)"}</span><span class="val">{format_value(result.rho, precision, "кг/м^3")}</span></div>
                <div class="result-item"><span class="label">{"Энтальпия (h)"}</span><span class="val">{format_value(result.h, precision, "кДж/кг")}</span></div>
                <div class="result-item"><span class="label">{"Энтропия (s)"}</span><span class="val">{format_value(result.s, precision, "кДж/кгК")}</span></div>
                <div class="result-item"><span class="label">{"Внутренняя энергия (u)"}</span><span class="val">{format_value(result.u, precision, "кДж/кг")}</span></div>
                <div class="result-item"><span class="label">{"Изобарная теплоемкость (cp)"}</span><span class="val">{format_value(result.cp, precision, "кДж/кгК")}</span></div>
                <div class="result-item"><span class="label">{"Скорость звука (w)"}</span><span class="val">{format_value(result.w, precision, "м/с")}</span></div>
                <div class="result-item region-highlight"><span class="label">{"Регион"}</span><span class="val">{result.region.clone()}</span></div>
            </div>
        </div>
    }
}

#[function_component(SingleCalcTab)]
pub fn single_calc_tab() -> Html {
    let state_ctx = use_context::<StateContext>().expect("Контекст состояния не найден");
    let app_ctx = use_context::<AppContext>().expect("Контекст данных не найден");

    let custom_name = use_state(String::new);
    let save_status = use_state(|| Option::<String>::None);
    let request_seq = use_mut_ref(|| 0_u64);

    let s = &*state_ctx;
    let is_already_saved = is_saved_request(&app_ctx, s.s_mode, &s.s_v1, &s.s_v2);
    let (label_v1, label_v2) = labels_for_mode(s.s_mode);

    use_effect_with((s.s_mode, s.s_v1.clone(), s.s_v2.clone()), {
        let state_ctx = state_ctx.clone();
        let app_ctx = app_ctx.clone();
        let custom_name = custom_name.clone();
        let request_seq = request_seq.clone();
        move |_| {
            run_single_calculation(
                (*state_ctx).clone(),
                state_ctx.clone(),
                app_ctx.clone(),
                custom_name.clone(),
                request_seq.clone(),
            );
            || ()
        }
    });

    let on_mode_change = {
        let state_ctx = state_ctx.clone();
        let save_status = save_status.clone();
        Callback::from(move |event: Event| {
            if let Some(select) = event.target_dyn_into::<HtmlSelectElement>() {
                let mut next = (*state_ctx).clone();
                next.s_mode = match select.value().as_str() {
                    "pt" => InputMode::Pt,
                    "ph" => InputMode::Ph,
                    "ps" => InputMode::Ps,
                    "px" => InputMode::Px,
                    "rhot" => InputMode::Rhot,
                    _ => InputMode::Pt,
                };
                state_ctx.set(next);
                save_status.set(None);
            }
        })
    };

    let on_precision_change = {
        let state_ctx = state_ctx.clone();
        Callback::from(move |event: Event| {
            if let Some(select) = event.target_dyn_into::<HtmlSelectElement>() {
                if let Ok(precision) = select.value().parse::<usize>() {
                    let mut next = (*state_ctx).clone();
                    next.s_precision = precision.min(10);
                    state_ctx.set(next);
                }
            }
        })
    };

    let on_v1_input = {
        let state_ctx = state_ctx.clone();
        let save_status = save_status.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
                let mut next = (*state_ctx).clone();
                next.s_v1 = input.value();
                state_ctx.set(next);
                save_status.set(None);
            }
        })
    };

    let on_v2_input = {
        let state_ctx = state_ctx.clone();
        let save_status = save_status.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
                let mut next = (*state_ctx).clone();
                next.s_v2 = input.value();
                state_ctx.set(next);
                save_status.set(None);
            }
        })
    };

    let on_name_input = {
        let custom_name = custom_name.clone();
        let save_status = save_status.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
                custom_name.set(input.value());
                save_status.set(None);
            }
        })
    };

    let on_save = {
        let state_ctx = state_ctx.clone();
        let app_ctx = app_ctx.clone();
        let custom_name = custom_name.clone();
        let save_status = save_status.clone();
        Callback::from(move |_| {
            let Some(result) = state_ctx.s_res.clone() else {
                return;
            };

            let mut saved = (*app_ctx).clone();
            let request_name = if custom_name.trim().is_empty() {
                format!(
                    "Точка {}",
                    saved
                        .items
                        .iter()
                        .filter(|item| matches!(item, SavedItem::Point(_)))
                        .count()
                        + 1
                )
            } else {
                (*custom_name).clone()
            };

            if let Some(SavedItem::Point(existing)) = saved.items.iter_mut().find(|item| match item {
                SavedItem::Point(point) => {
                    matching_request(state_ctx.s_mode, &state_ctx.s_v1, &state_ctx.s_v2, point)
                }
                SavedItem::Table(_) => false,
            }) {
                existing.name = request_name.clone();
                existing.state = result;
                existing.orig_mode = state_ctx.s_mode;
                existing.orig_v1 = state_ctx.s_v1.clone();
                existing.orig_v2 = state_ctx.s_v2.clone();
                save_status.set(Some("Обновлено".to_string()));
            } else {
                let id = saved.allocate_id();
                saved.items.push(SavedItem::Point(SavedPoint {
                    id,
                    name: request_name.clone(),
                    state: result,
                    orig_mode: state_ctx.s_mode,
                    orig_v1: state_ctx.s_v1.clone(),
                    orig_v2: state_ctx.s_v2.clone(),
                }));
                save_status.set(Some("Сохранено".to_string()));
            }

            custom_name.set(request_name);
            app_ctx.set(saved);
        })
    };

    html! {
        <div style="position: relative; height: 100%; overflow: hidden; display: flex; flex-direction: column;">
            <div class="tab-content fade-in" style="flex-grow: 1; overflow-y: auto; padding-bottom: 20px;">
                <div class="card instruction-card">
                    <h2 style="margin: 0; margin-bottom: 5px;">{"Одиночный расчет"}</h2>
                </div>

                <div class="card">
                    <div class="input-group">
                        <label>{"Пара известного ввода:"}</label>
                        <select class="styled-select" value={s.s_mode.as_str()} onchange={on_mode_change}>
                            <option value="pt">{"p-T (давление и температура)"}</option>
                            <option value="rhot">{"rho-T (плотность и температура)"}</option>
                            <option value="px">{"p-x (давление и степень сухости)"}</option>
                            <option value="ps">{"p-s (давление и энтропия)"}</option>
                            <option value="ph">{"p-h (давление и энтальпия)"}</option>
                        </select>
                    </div>
                    <div class="input-group" style="max-width: 220px;">
                        <label>{"Точность вывода (знаков):"}</label>
                        <select class="styled-select" value={s.s_precision.to_string()} onchange={on_precision_change}>
                            { for (0..=10).map(|precision| html! {
                                <option value={precision.to_string()} selected={precision == s.s_precision}>{precision}</option>
                            }) }
                        </select>
                    </div>
                    <div class="input-row">
                        <div class="input-group">
                            <label>{label_v1}</label>
                            <input type="text" class="styled-input" value={s.s_v1.clone()} oninput={on_v1_input} placeholder="Ввод..." />
                        </div>
                        <div class="input-group">
                            <label>{label_v2}</label>
                            <input type="text" class="styled-input" value={s.s_v2.clone()} oninput={on_v2_input} placeholder="Ввод..." />
                        </div>
                    </div>
                    <div class="actions" style="align-items: center; background: var(--hover-bg); padding: 15px; border-radius: 8px; border: 1px solid var(--border); margin-top: 10px;">
                        <div style="display: flex; gap: 10px; flex-grow: 1; align-items: center;">
                            <input type="text" class="styled-input" style="flex-grow: 1;" value={(*custom_name).clone()} oninput={on_name_input} placeholder="Имя сохраненной точки..." disabled={s.s_res.is_none()} />
                            <button class={classes!("btn", if is_already_saved { "btn-primary" } else { "btn-success" })} onclick={on_save} disabled={s.s_res.is_none()} style={if s.s_res.is_none() { "opacity: 0.5; cursor: not-allowed; white-space: nowrap;" } else { "white-space: nowrap;" }}>
                                { if is_already_saved { "Обновить" } else { "Сохранить" } }
                            </button>
                        </div>
                        { if let Some(status) = &*save_status { html! { <div class="fade-in" style="color: #198754; font-weight: 500; font-size: 0.95rem; margin-left: 15px; min-width: 120px;">{ status }</div> } } else { html! { <div style="min-width: 120px; margin-left: 15px;"></div> } } }
                    </div>
                </div>

                { if let Some(error) = &s.s_error {
                    html! { <div class="card fade-in" style="border-color: #dc3545; background-color: #f8d7da; color: #842029;"><strong>{"Ошибка расчета: "}</strong>{error}</div> }
                } else {
                    html! {}
                }}

                { s.s_res.as_ref().map(|result| view_result(result, s.s_precision)).unwrap_or_default() }
            </div>

            <div style={format!("position: absolute; top: 0; left: 0; bottom: 0; width: 320px; background: var(--card-bg); border-right: 1px solid var(--border); box-shadow: 4px 0 15px rgba(0,0,0,0.15); transform: translateX({}); transition: transform 0.3s cubic-bezier(0.4, 0.0, 0.2, 1); display: flex; flex-direction: column; z-index: 50;", if s.right_sidebar_open { "0" } else { "-120%" })}>
                <div style="padding: 15px; border-bottom: 1px solid var(--border); background: var(--hover-bg);">
                    <h3 style="margin: 0; font-size: 1.1rem;">{"Сохранённые точки"}</h3>
                </div>
                <div style="padding: 15px; overflow-y: auto; flex-grow: 1; display: flex; flex-direction: column; gap: 8px;">
                    { if !app_ctx.items.iter().any(|item| matches!(item, SavedItem::Point(_))) { html! { <div style="color: var(--text-muted); font-size: 0.9rem; text-align: center; margin-top: 20px;">{"Нет сохранённых точек"}</div> } } else { html! {} } }

                    { for app_ctx.items.iter().filter_map(|item| if let SavedItem::Point(point) = item { Some(point) } else { None }).map(|point| {
                        let on_load = {
                            let state_ctx = state_ctx.clone();
                            let custom_name = custom_name.clone();
                            let point = point.clone();
                            Callback::from(move |_| {
                                let mut next = (*state_ctx).clone();
                                next.s_mode = point.orig_mode;
                                next.s_v1 = point.orig_v1.clone();
                                next.s_v2 = point.orig_v2.clone();
                                next.s_res = Some(point.state.clone());
                                next.s_error = None;
                                state_ctx.set(next);
                                custom_name.set(point.name.clone());
                            })
                        };
                        let on_delete = {
                            let app_ctx = app_ctx.clone();
                            let state_ctx = state_ctx.clone();
                            let point_id = point.id;
                            Callback::from(move |event: MouseEvent| {
                                event.stop_propagation();
                                let mut saved = (*app_ctx).clone();
                                saved.items.retain(|item| item.id() != point_id);
                                app_ctx.set(saved);

                                let mut next = (*state_ctx).clone();
                                next.plot_selected.remove(&point_id);
                                state_ctx.set(next);
                            })
                        };

                        html! {
                            <div style="display: flex; justify-content: space-between; align-items: center; padding: 10px; background: var(--bg-color); border: 1px solid var(--border); border-radius: 6px; cursor: pointer;" onclick={on_load}>
                                <span style="font-weight: 500; font-size: 0.95rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">{ &point.name }</span>
                                <button class="btn btn-sm" style="background: transparent; border: none; color: #dc3545; padding: 5px; font-size: 1.1rem; cursor: pointer;" onclick={on_delete} title="Удалить">{"X"}</button>
                            </div>
                        }
                    }) }
                </div>
            </div>
        </div>
    }
}
