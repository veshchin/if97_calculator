/* File: src/ui/table_calc.rs */
use yew::prelude::*;
use web_sys::{HtmlTextAreaElement, HtmlSelectElement, HtmlInputElement, MouseEvent};
use if97_core::domain::calculator::If97;
use if97_core::domain::state::WaterState;
use crate::types::{StateContext, AppContext, SavedTable, SavedItem, PersistentState};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    pub async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[derive(serde::Serialize)] struct EmptyArgs {}
#[derive(serde::Serialize)] struct SaveArgs { content: String, default_name: Option<String>, filter_name: Option<String>, filter_ext: Option<String> }

fn is_same_state(s1: &WaterState, s2: &WaterState) -> bool {
    s1.p.inner() == s2.p.inner() && s1.t.inner() == s2.t.inner() && s1.v.inner() == s2.v.inner() && s1.h.inner() == s2.h.inner() && s1.s.inner() == s2.s.inner()
}

fn is_same_table(t1: &[WaterState], t2: &[WaterState]) -> bool {
    t1.len() == t2.len() && t1.iter().zip(t2.iter()).all(|(s1, s2)| is_same_state(s1, s2))
}

fn calculate_table(mode: &str, input: &str) -> Vec<Result<WaterState, String>> {
    let mut results = Vec::new();
    for line in input.lines() {
        let line = line.trim(); if line.is_empty() { continue; }
        let parts: Vec<&str> = line.split(|c: char| c == ';' || c == '\t' || c.is_whitespace() || c == ',').filter(|s| !s.is_empty()).collect();
        if parts.len() >= 2 {
            let v1_str = parts[0].replace(',', "."); let v2_str = parts[1].replace(',', ".");
            if let (Ok(v1), Ok(v2)) = (v1_str.parse::<f64>(), v2_str.parse::<f64>()) {
                let res = match mode {
                    "pt" => If97::pt(v1.into(), v2.into()), "ph" => If97::ph(v1.into(), v2.into()),
                    "ps" => If97::ps(v1.into(), v2.into()), "px" => If97::px(v1.into(), v2.into()),
                    "rhot" => If97::rhot(v1.into(), v2.into()), _ => Err(if97_core::domain::errors::If97Error::InvalidInput("".into())),
                };
                results.push(res.map_err(|_| "Вне диапазона".to_string()));
            } else { results.push(Err("Ошибка чтения".to_string())); }
        } else { results.push(Err("Мало колонок".to_string())); }
    }
    results
}

fn process_table_state(mut new_state: PersistentState, app_ctx: &Vec<SavedItem>, custom_name: &UseStateHandle<String>) -> PersistentState {
    let results = calculate_table(&new_state.t_mode, &new_state.t_input);
    new_state.t_res = results.clone();
    let valid_states: Vec<WaterState> = results.iter().filter_map(|r| r.clone().ok()).collect();

    if !valid_states.is_empty() {
        if let Some(SavedItem::Table(existing)) = app_ctx.iter().find(|i| {
            if let SavedItem::Table(t) = i { is_same_table(&t.states, &valid_states) } else { false }
        }) {
            custom_name.set(existing.name.clone());
        } else if app_ctx.iter().any(|i| if let SavedItem::Table(t) = i { t.name == **custom_name } else { false }) && !custom_name.is_empty() {
            custom_name.set(String::new());
        }
    } else if !custom_name.is_empty() {
        custom_name.set(String::new());
    }
    new_state
}

#[function_component(TableCalcTab)]
pub fn table_calc_tab() -> Html {
    let state_ctx = use_context::<StateContext>().expect("No Context");
    let app_ctx = use_context::<AppContext>().expect("No Context");

    let s = &*state_ctx;
    let custom_name = use_state(String::new);
    let save_status = use_state(|| Option::<String>::None);
    let is_sidebar_open = use_state(|| false);

    let on_mode = {
        let state_ctx = state_ctx.clone(); let app_ctx = app_ctx.clone(); let custom_name = custom_name.clone(); let save_status = save_status.clone();
        Callback::from(move |e: Event| {
            if let Some(select) = e.target_dyn_into::<HtmlSelectElement>() {
                let mut new_s = (*state_ctx).clone(); new_s.t_mode = select.value();
                state_ctx.set(process_table_state(new_s, &app_ctx, &custom_name)); save_status.set(None);
            }
        })
    };

    let on_input = {
        let state_ctx = state_ctx.clone(); let app_ctx = app_ctx.clone(); let custom_name = custom_name.clone(); let save_status = save_status.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(textarea) = e.target_dyn_into::<HtmlTextAreaElement>() {
                let mut new_s = (*state_ctx).clone(); new_s.t_input = textarea.value();
                state_ctx.set(process_table_state(new_s, &app_ctx, &custom_name)); save_status.set(None);
            }
        })
    };

    let on_name_input = {
        let custom_name = custom_name.clone(); let save_status = save_status.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target_dyn_into::<HtmlInputElement>() { custom_name.set(input.value()); save_status.set(None); }
        })
    };

    let on_load_file = {
        let state_ctx = state_ctx.clone(); let app_ctx = app_ctx.clone(); let custom_name = custom_name.clone(); let save_status = save_status.clone();
        Callback::from(move |_| {
            let state_ctx = state_ctx.clone(); let app_ctx = app_ctx.clone(); let custom_name = custom_name.clone(); let save_status = save_status.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(args) = serde_wasm_bindgen::to_value(&EmptyArgs {}) {
                    if let Ok(js_res) = invoke("load_file_dialog", args).await {
                        if let Some(content) = js_res.as_string() {
                            let mut new_s = (*state_ctx).clone(); new_s.t_input = content.clone();
                            state_ctx.set(process_table_state(new_s, &app_ctx, &custom_name)); save_status.set(None);
                        }
                    }
                }
            });
        })
    };

    let on_export_csv = {
        let state_ctx = state_ctx.clone();
        Callback::from(move |_| {
            let s = &*state_ctx; if s.t_res.is_empty() { return; }
            let mut csv = String::from("p (MPa);T (K);v (m3/kg);rho (kg/m3);h (kJ/kg);s (kJ/kgK);u (kJ/kg);cp (kJ/kgK);w (m/s);Region\n");
            for res in &s.t_res {
                match res {
                    Ok(st) => { csv.push_str(&format!("{:.6};{:.2};{:.6};{:.4};{:.4};{:.4};{:.4};{:.4};{:.2};{:?}\n", st.p.inner(), st.t.inner(), st.v.inner(), st.rho.inner(), st.h.inner(), st.s.inner(), st.u.inner(), st.cp.inner(), st.w.inner(), st.region)); },
                    Err(e) => { csv.push_str(&format!("Error: {};;;;;;;;;\n", e)); }
                }
            }
            wasm_bindgen_futures::spawn_local(async move {
                let args = SaveArgs { content: csv, default_name: Some("table_export.csv".to_string()), filter_name: Some("CSV Документ".to_string()), filter_ext: Some("csv".to_string()) };
                if let Ok(js_args) = serde_wasm_bindgen::to_value(&args) { let _ = invoke("save_file_dialog", js_args).await; }
            });
        })
    };

    let on_save_to_plots = {
        let state_ctx = state_ctx.clone(); let app_ctx = app_ctx.clone(); let custom_name = custom_name.clone(); let save_status = save_status.clone();
        Callback::from(move |_| {
            let s = &*state_ctx; let mut items = (*app_ctx).clone();
            let valid_states: Vec<WaterState> = s.t_res.iter().filter_map(|r| r.clone().ok()).collect();
            if valid_states.is_empty() { return; }

            let target_name = if custom_name.trim().is_empty() {
                let count = items.iter().filter(|i| matches!(i, SavedItem::Table(_))).count();
                format!("Таблица {}", count + 1)
            } else { (*custom_name).clone() };

            let mut updated = false;

            if let Some(SavedItem::Table(existing)) = items.iter_mut().find(|i| {
                if let SavedItem::Table(t) = i { is_same_table(&t.states, &valid_states) } else { false }
            }) {
                existing.name = if custom_name.trim().is_empty() { existing.name.clone() } else { target_name.clone() };
                existing.orig_mode = s.t_mode.clone();
                existing.orig_input = s.t_input.clone();
                updated = true;
                if custom_name.trim().is_empty() { custom_name.set(existing.name.clone()); }
            } else {
                items.push(SavedItem::Table(SavedTable {
                    name: target_name, states: valid_states,
                    orig_mode: s.t_mode.clone(), orig_input: s.t_input.clone()
                }));
                custom_name.set(String::new());
            }

            app_ctx.set(items);
            save_status.set(Some(if updated { "✅ Обновлено" } else { "✅ Сохранено" }.to_string()));
        })
    };

    let toggle_sidebar = { let s = is_sidebar_open.clone(); Callback::from(move |_| s.set(!*s)) };

    let valid_states: Vec<WaterState> = s.t_res.iter().filter_map(|r| r.clone().ok()).collect();
    let has_valid_results = !valid_states.is_empty();

    let is_already_saved = if has_valid_results {
        app_ctx.iter().any(|i| if let SavedItem::Table(t) = i { is_same_table(&t.states, &valid_states) } else { false })
    } else { false };

    html! {
        <div style="position: relative; height: 100%; overflow: hidden; display: flex; flex-direction: column;">
            <div class="table-calc-container fade-in" style="flex-grow: 1; overflow-y: auto; padding-bottom: 20px; display: flex; flex-direction: column;">

                <div class="card instruction-card" style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 15px;">
                    <div>
                        <h2 style="margin: 0; margin-bottom: 5px;">{"Табличный расчет"}</h2>
                        <p class="text-muted" style="margin: 0;">{"Вставьте столбцы с данными из Excel/TXT или загрузите файл. Разделители определяются автоматически."}</p>
                    </div>
                    <button class="btn btn-primary" onclick={toggle_sidebar.clone()}>
                        { if *is_sidebar_open { "Скрыть панель ➡" } else { "📂 Сохраненные точки" } }
                    </button>
                </div>

                <div class="toolbar">
                    <div style="display: flex; gap: 10px; align-items: center;">
                        <label style="font-weight: 500;">{"Режим:"}</label>
                        <select class="styled-select" value={s.t_mode.clone()} onchange={on_mode} style="min-width: 150px;">
                            <option value="pt">{"p-T"}</option><option value="ph">{"p-h"}</option><option value="ps">{"p-s"}</option><option value="px">{"p-x"}</option><option value="rhot">{"rho-T"}</option>
                        </select>
                    </div>
                    <button class="btn btn-outline" onclick={on_load_file}>{"📂 Открыть"}</button>
                    <div class="spacer"></div>
                    <div style="display: flex; gap: 10px; align-items: center; background: var(--hover-bg); padding: 5px 10px; border-radius: 6px; border: 1px solid var(--border);">
                        <input type="text" class="styled-input" style="width: 220px; padding: 6px 10px;" value={(*custom_name).clone()} oninput={on_name_input} placeholder="Имя для этой таблицы..." disabled={!has_valid_results} />
                        <button class={classes!("btn", "btn-sm", if is_already_saved { "btn-primary" } else { "btn-success" })} onclick={on_save_to_plots} disabled={!has_valid_results} style={if !has_valid_results { "opacity: 0.5; cursor: not-allowed;" } else { "" }}>
                            { if is_already_saved { "🔄 Обновить" } else { "Сохранить 📈" } }
                        </button>
                        <button class="btn btn-success btn-sm" onclick={on_export_csv} disabled={s.t_res.is_empty()} style={if s.t_res.is_empty() { "opacity: 0.5; cursor: not-allowed;" } else { "" }}>{"Экспорт CSV 💾"}</button>
                        { if let Some(status) = &*save_status { html! { <span class="fade-in" style="color: #198754; font-size: 0.9rem; font-weight: 500;">{ status }</span> } } else { html! {} } }
                    </div>
                </div>
                <div class="split-view" style="margin-top: 15px; flex-grow: 1;">
                    <div class="split-left" style="width: 20%;"><textarea class="raw-data-area" oninput={on_input} value={s.t_input.clone()} placeholder="Ввод данных..."></textarea></div>
                    <div class="split-right" style="width: 80%; overflow-x: auto;">
                        <div class="table-container">
                            <table class="data-table" style="white-space: nowrap;">
                                <thead><tr><th style="width: 40px;">{"#"}</th><th>{"p (MPa)"}</th><th>{"T (K)"}</th><th>{"v (m³/kg)"}</th><th>{"rho"}</th><th>{"h (kJ/kg)"}</th><th>{"s (kJ/kgK)"}</th><th>{"Region"}</th></tr></thead>
                                <tbody>
                                    { for s.t_res.iter().enumerate().map(|(i, r)| match r { Ok(st) => html! { <tr><td>{i + 1}</td><td>{format!("{:.4}", st.p.inner())}</td><td>{format!("{:.2}", st.t.inner())}</td><td>{format!("{:.6}", st.v.inner())}</td><td>{format!("{:.4}", st.rho.inner())}</td><td>{format!("{:.4}", st.h.inner())}</td><td>{format!("{:.4}", st.s.inner())}</td><td>{format!("{:?}", st.region)}</td></tr> }, Err(e) => html! { <tr><td>{i + 1}</td><td colspan="7">{format!("Ошибка: {}", e)}</td></tr> } }) }
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>
            </div>

            // ВЫЕЗЖАЮЩАЯ ШТОРКА
            <div style={format!("position: absolute; top: 0; right: 0; bottom: 0; width: 320px; background: var(--card-bg); border-left: 1px solid var(--border); box-shadow: -4px 0 15px rgba(0,0,0,0.1); transform: translateX({}); transition: transform 0.3s cubic-bezier(0.4, 0.0, 0.2, 1); display: flex; flex-direction: column; z-index: 10;", if *is_sidebar_open { "0" } else { "100%" })}>
                <div style="padding: 15px; border-bottom: 1px solid var(--border); display: flex; justify-content: space-between; align-items: center; background: var(--hover-bg);">
                    <h3 style="margin: 0; font-size: 1.1rem;">{"Сохраненные таблицы"}</h3>
                    <button class="btn btn-outline btn-sm" style="border: none;" onclick={toggle_sidebar}>{"❌"}</button>
                </div>
                <div style="padding: 15px; overflow-y: auto; flex-grow: 1; display: flex; flex-direction: column; gap: 8px;">
                    { if !app_ctx.iter().any(|i| matches!(i, SavedItem::Table(_))) { html! { <div style="color: var(--text-muted); font-size: 0.9rem; text-align: center; margin-top: 20px;">{"Нет сохраненных таблиц"}</div> } } else { html! {} } }

                    { for app_ctx.iter().filter_map(|i| if let SavedItem::Table(t) = i { Some(t) } else { None }).map(|t| {
                        let on_load = {
                            let state_ctx = state_ctx.clone(); let app_ctx_c = app_ctx.clone(); let custom_name = custom_name.clone(); let save_status = save_status.clone();
                            let tb = t.clone();
                            Callback::from(move |_| {
                                let mut s = (*state_ctx).clone();
                                s.t_mode = tb.orig_mode.clone(); s.t_input = tb.orig_input.clone();
                                state_ctx.set(process_table_state(s, &app_ctx_c, &custom_name));
                                custom_name.set(tb.name.clone());
                                save_status.set(Some("✅ Загружено".to_string()));
                            })
                        };
                        let on_delete = {
                            let app_ctx = app_ctx.clone(); let name = t.name.clone();
                            Callback::from(move |e: MouseEvent| {
                                e.stop_propagation();
                                let mut items = (*app_ctx).clone();
                                items.retain(|i| if let SavedItem::Table(st) = i { st.name != name } else { true });
                                app_ctx.set(items);
                            })
                        };
                        html! {
                            <div style="display: flex; justify-content: space-between; align-items: center; padding: 10px; background: var(--bg-color); border: 1px solid var(--border); border-radius: 6px; cursor: pointer;" onclick={on_load}>
                                <span style="font-weight: 500; font-size: 0.95rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">{ &t.name }</span>
                                <button class="btn btn-sm" style="background: transparent; border: none; color: #dc3545; padding: 5px; font-size: 1.1rem; cursor: pointer;" onclick={on_delete} title="Удалить">{"🗑️"}</button>
                            </div>
                        }
                    }) }
                </div>
            </div>

        </div>
    }
}