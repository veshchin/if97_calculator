/* File: src/ui/single_calc.rs */
use yew::prelude::*;
use web_sys::{HtmlInputElement, HtmlSelectElement, MouseEvent};
use if97_core::If97;
use if97_core::WaterState;
use crate::types::{StateContext, AppContext, SavedPoint, SavedItem, PersistentState};

fn is_same_state(s1: &WaterState, s2: &WaterState) -> bool {
    s1.p.inner() == s2.p.inner() && s1.t.inner() == s2.t.inner() && s1.v.inner() == s2.v.inner() && s1.h.inner() == s2.h.inner() && s1.s.inner() == s2.s.inner()
}

fn calculate_state(mode: &str, v1_str: &str, v2_str: &str) -> Result<WaterState, String> {
    if v1_str.trim().is_empty() || v2_str.trim().is_empty() { return Err("".to_string()); }
    let v1 = v1_str.trim().replace(',', ".").parse::<f64>().map_err(|_| "Ошибка парсинга".to_string())?;
    let v2 = v2_str.trim().replace(',', ".").parse::<f64>().map_err(|_| "Ошибка парсинга".to_string())?;

    match mode {
        "pt" => If97::pt(v1.into(), v2.into()), "ph" => If97::ph(v1.into(), v2.into()),
        "ps" => If97::ps(v1.into(), v2.into()), "px" => If97::px(v1.into(), v2.into()),
        "rhot" => If97::rhot(v1.into(), v2.into()), _ => Err(if97_core::errors::If97Error::InvalidInput("".into())),
    }.map_err(|e| match e {
        if97_core::errors::If97Error::PhaseBoundaryError(msg) => format!("Линия насыщения: {}", msg),
        if97_core::errors::If97Error::OutOfBounds(msg) => format!("Вне диапазона: {}", msg),
        _ => e.to_string(),
    })
}

fn process_single_state(mut new_state: PersistentState, app_ctx: &Vec<SavedItem>, custom_name: &UseStateHandle<String>) -> PersistentState {
    match calculate_state(&new_state.s_mode, &new_state.s_v1, &new_state.s_v2) {
        Ok(state) => {
            new_state.s_res = Some(state.clone()); new_state.s_error = None;
            if let Some(SavedItem::Point(existing)) = app_ctx.iter().find(|i| if let SavedItem::Point(sp) = i { is_same_state(&sp.state, &state) } else { false }) {
                custom_name.set(existing.name.clone());
            } else if app_ctx.iter().any(|i| if let SavedItem::Point(sp) = i { sp.name == **custom_name } else { false }) {
                custom_name.set(String::new());
            }
        },
        Err(err) => {
            new_state.s_res = None;
            new_state.s_error = if err.is_empty() { None } else { Some(err) };
            if app_ctx.iter().any(|i| if let SavedItem::Point(sp) = i { sp.name == **custom_name } else { false }) { custom_name.set(String::new()); }
        }
    }
    new_state
}

#[function_component(SingleCalcTab)]
pub fn single_calc_tab() -> Html {
    let state_ctx = use_context::<StateContext>().expect("No StateContext found");
    let app_ctx = use_context::<AppContext>().expect("No AppContext found");

    let s = &*state_ctx;
    let custom_name = use_state(String::new);
    let save_status = use_state(|| Option::<String>::None);

    let is_already_saved = s.s_res.as_ref().map_or(false, |res| {
        app_ctx.iter().any(|i| if let SavedItem::Point(sp) = i { is_same_state(&sp.state, res) } else { false })
    });

    let on_mode_change = {
        let state_ctx = state_ctx.clone(); let app_ctx = app_ctx.clone();
        let custom_name = custom_name.clone(); let save_status = save_status.clone();
        Callback::from(move |e: Event| {
            if let Some(select) = e.target_dyn_into::<HtmlSelectElement>() {
                let mut new_state = (*state_ctx).clone(); new_state.s_mode = select.value();
                state_ctx.set(process_single_state(new_state, &app_ctx, &custom_name)); save_status.set(None);
            }
        })
    };

    let on_v1_input = {
        let state_ctx = state_ctx.clone(); let app_ctx = app_ctx.clone();
        let custom_name = custom_name.clone(); let save_status = save_status.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                let mut new_state = (*state_ctx).clone(); new_state.s_v1 = input.value();
                state_ctx.set(process_single_state(new_state, &app_ctx, &custom_name)); save_status.set(None);
            }
        })
    };

    let on_v2_input = {
        let state_ctx = state_ctx.clone(); let app_ctx = app_ctx.clone();
        let custom_name = custom_name.clone(); let save_status = save_status.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() {
                let mut new_state = (*state_ctx).clone(); new_state.s_v2 = input.value();
                state_ctx.set(process_single_state(new_state, &app_ctx, &custom_name)); save_status.set(None);
            }
        })
    };

    let on_name_input = {
        let custom_name = custom_name.clone(); let save_status = save_status.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() { custom_name.set(input.value()); save_status.set(None); }
        })
    };

    let on_save = {
        let state_ctx = state_ctx.clone(); let app_ctx = app_ctx.clone();
        let custom_name = custom_name.clone(); let save_status = save_status.clone();
        Callback::from(move |_| {
            if let Some(res) = &state_ctx.s_res {
                let s_ref = &*state_ctx;
                let mut items = (*app_ctx).clone();
                let target_name = if custom_name.trim().is_empty() { format!("Точка {}", items.iter().filter(|i| matches!(i, SavedItem::Point(_))).count() + 1) } else { (*custom_name).clone() };

                if let Some(SavedItem::Point(existing)) = items.iter_mut().find(|i| if let SavedItem::Point(sp) = i { is_same_state(&sp.state, res) } else { false }) {
                    existing.name = if custom_name.trim().is_empty() { existing.name.clone() } else { target_name.clone() };
                    existing.orig_mode = s_ref.s_mode.clone();
                    existing.orig_v1 = s_ref.s_v1.clone();
                    existing.orig_v2 = s_ref.s_v2.clone();
                    save_status.set(Some("✅ Обновлено".to_string()));
                    if custom_name.trim().is_empty() { custom_name.set(existing.name.clone()); }
                } else {
                    items.push(SavedItem::Point(SavedPoint {
                        name: target_name, state: res.clone(),
                        orig_mode: s_ref.s_mode.clone(), orig_v1: s_ref.s_v1.clone(), orig_v2: s_ref.s_v2.clone()
                    }));
                    save_status.set(Some("✅ Сохранено".to_string()));
                    custom_name.set(String::new());
                }
                app_ctx.set(items);
            }
        })
    };

    html! {
        <div style="position: relative; height: 100%; overflow: hidden; display: flex; flex-direction: column;">
            <div class="tab-content fade-in" style="flex-grow: 1; overflow-y: auto; padding-bottom: 20px;">
                <div class="card instruction-card">
                    <h2 style="margin: 0; margin-bottom: 5px;">{"Одиночный расчет"}</h2>
                    <p class="text-muted" style="margin: 0;">{"Мгновенный расчет свойств воды."}</p>
                </div>

                <div class="card">
                    <div class="input-group">
                        <label>{"Что известно:"}</label>
                        <select class="styled-select" value={s.s_mode.clone()} onchange={on_mode_change}>
                            <option value="pt">{"p-T (Давление и Температура)"}</option>
                            <option value="rhot">{"rho-T (Плотность и Температура)"}</option>
                            <option value="px">{"p-x (Давление и Степень сухости)"}</option>
                            <option value="ps">{"p-s (Давление и Энтропия)"}</option>
                            <option value="ph">{"p-h (Давление и Энтальпия)"}</option>
                        </select>
                    </div>
                    <div class="input-row">
                        <div class="input-group"><input type="text" class="styled-input" value={s.s_v1.clone()} oninput={on_v1_input} placeholder="Ввод..." /></div>
                        <div class="input-group"><input type="text" class="styled-input" value={s.s_v2.clone()} oninput={on_v2_input} placeholder="Ввод..." /></div>
                    </div>
                    <div class="actions" style="align-items: center; background: var(--hover-bg); padding: 15px; border-radius: 8px; border: 1px solid var(--border); margin-top: 10px;">
                        <div style="display: flex; gap: 10px; flex-grow: 1; align-items: center;">
                            <input type="text" class="styled-input" style="flex-grow: 1;" value={(*custom_name).clone()} oninput={on_name_input} placeholder="Свое имя точки (необязательно)..." disabled={s.s_res.is_none()} />
                            <button class={classes!("btn", if is_already_saved { "btn-primary" } else { "btn-success" })} onclick={on_save} disabled={s.s_res.is_none()} style={if s.s_res.is_none() { "opacity: 0.5; cursor: not-allowed; white-space: nowrap;" } else { "white-space: nowrap;" }}>
                                { if is_already_saved { "Обновить" } else { "Сохранить" } }
                            </button>
                        </div>
                        { if let Some(status) = &*save_status { html! { <div class="fade-in" style="color: #198754; font-weight: 500; font-size: 0.95rem; margin-left: 15px; min-width: 120px;">{ status }</div> } } else { html! { <div style="min-width: 120px; margin-left: 15px;"></div> } } }
                    </div>
                </div>
                { if let Some(err_msg) = &s.s_error { html! { <div class="card fade-in" style="border-color: #dc3545; background-color: #f8d7da; color: #842029;"><strong>{"Ошибка расчета: "}</strong> { err_msg }</div> } } else { html! {} } }
                { if let Some(res) = &s.s_res {
                    html! {
                        <div class="card fade-in">
                            <h3>{"Результаты расчета"}</h3>
                            <div class="results-grid">
                                <div class="result-item"><span class="label">{"Давление (p)"}</span><span class="val">{format!("{:.4} MPa", res.p.inner())}</span></div>
                                <div class="result-item"><span class="label">{"Температура (T)"}</span><span class="val">{format!("{:.2} K", res.t.inner())}</span></div>
                                <div class="result-item"><span class="label">{"Уд. объем (v)"}</span><span class="val">{format!("{:.6} m³/kg", res.v.inner())}</span></div>
                                <div class="result-item"><span class="label">{"Плотность (rho)"}</span><span class="val">{format!("{:.4} kg/m³", res.rho.inner())}</span></div>
                                <div class="result-item"><span class="label">{"Энтальпия (h)"}</span><span class="val">{format!("{:.4} kJ/kg", res.h.inner())}</span></div>
                                <div class="result-item"><span class="label">{"Энтропия (s)"}</span><span class="val">{format!("{:.4} kJ/kgK", res.s.inner())}</span></div>
                                <div class="result-item"><span class="label">{"Внутр. энергия (u)"}</span><span class="val">{format!("{:.4} kJ/kg", res.u.inner())}</span></div>
                                <div class="result-item"><span class="label">{"Изобар. теплоемк. (cp)"}</span><span class="val">{format!("{:.4} kJ/kgK", res.cp.inner())}</span></div>
                                <div class="result-item"><span class="label">{"Скорость звука (w)"}</span><span class="val">{format!("{:.2} m/s", res.w.inner())}</span></div>
                                <div class="result-item region-highlight"><span class="label">{"Регион"}</span><span class="val">{format!("{:?}", res.region)}</span></div>
                            </div>
                        </div>
                    }
                } else { html! {} } }
            </div>

            // OVERLAY ПАНЕЛЬ ДАННЫХ (Выезжает слева)
            <div style={format!("position: absolute; top: 0; left: 0; bottom: 0; width: 320px; background: var(--card-bg); border-right: 1px solid var(--border); box-shadow: 4px 0 15px rgba(0,0,0,0.15); transform: translateX({}); transition: transform 0.3s cubic-bezier(0.4, 0.0, 0.2, 1); display: flex; flex-direction: column; z-index: 50;", if s.right_sidebar_open { "0" } else { "-120%" })}>
                <div style="padding: 15px; border-bottom: 1px solid var(--border); background: var(--hover-bg);">
                    <h3 style="margin: 0; font-size: 1.1rem;">{"Сохраненные точки"}</h3>
                </div>
                <div style="padding: 15px; overflow-y: auto; flex-grow: 1; display: flex; flex-direction: column; gap: 8px;">
                    { if !app_ctx.iter().any(|i| matches!(i, SavedItem::Point(_))) { html! { <div style="color: var(--text-muted); font-size: 0.9rem; text-align: center; margin-top: 20px;">{"Нет сохраненных точек"}</div> } } else { html! {} } }

                    { for app_ctx.iter().filter_map(|i| if let SavedItem::Point(p) = i { Some(p) } else { None }).map(|p| {
                        let on_load = {
                            let state_ctx = state_ctx.clone(); let app_ctx_c = app_ctx.clone(); let custom_name = custom_name.clone(); let save_status = save_status.clone();
                            let pt = p.clone();
                            Callback::from(move |_| {
                                let mut s = (*state_ctx).clone();
                                s.s_mode = pt.orig_mode.clone(); s.s_v1 = pt.orig_v1.clone(); s.s_v2 = pt.orig_v2.clone();
                                state_ctx.set(process_single_state(s, &app_ctx_c, &custom_name));
                                custom_name.set(pt.name.clone());
                                save_status.set(Some("✅ Загружено".to_string()));
                            })
                        };
                        let on_delete = {
                            let app_ctx = app_ctx.clone();
                            let name = p.name.clone();
                            Callback::from(move |e: MouseEvent| {
                                e.stop_propagation();
                                let mut items = (*app_ctx).clone();
                                items.retain(|i| if let SavedItem::Point(sp) = i { sp.name != name } else { true });
                                app_ctx.set(items);
                            })
                        };
                        html! {
                            <div style="display: flex; justify-content: space-between; align-items: center; padding: 10px; background: var(--bg-color); border: 1px solid var(--border); border-radius: 6px; cursor: pointer;" onclick={on_load}>
                                <span style="font-weight: 500; font-size: 0.95rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">{ &p.name }</span>
                                <button class="btn btn-sm" style="background: transparent; border: none; color: #dc3545; padding: 5px; font-size: 1.1rem; cursor: pointer;" onclick={on_delete} title="Удалить">{"🗑️"}</button>
                            </div>
                        }
                    }) }
                </div>
            </div>
        </div>
    }
}