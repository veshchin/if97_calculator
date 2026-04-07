use crate::tauri_api;
use crate::types::{AppContext, SavedItem, SavedTable, StateContext};
use if97_app_api::{InputMode, StateDto, TableCalcRequest, TableRowResult};
use std::cell::RefCell;
use std::rc::Rc;
use web_sys::{HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement, MouseEvent};
use yew::prelude::*;

const MAX_VISIBLE_ROWS: usize = 300;
const MAX_GENERATED_ROWS: usize = 20_000;

fn format_value(value: f64, precision: usize) -> String {
    if value.is_infinite() {
        if value.is_sign_negative() {
            "-∞".to_string()
        } else {
            "∞".to_string()
        }
    } else if value.is_nan() {
        "NaN".to_string()
    } else {
        format!("{:.*}", precision, value)
    }
}

fn normalize_table_input(input: &str) -> String {
    input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn table_matches_request(mode: InputMode, input: &str, table: &SavedTable) -> bool {
    table.orig_mode == mode && normalize_table_input(&table.orig_input) == normalize_table_input(input)
}

fn valid_states(rows: &[TableRowResult]) -> Vec<StateDto> {
    rows.iter()
        .filter_map(|row| row.state.clone())
        .collect::<Vec<_>>()
}

fn rows_from_states(states: &[StateDto]) -> Rc<Vec<TableRowResult>> {
    Rc::new(
        states
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, state)| TableRowResult {
                line_no: index + 1,
                state: Some(state),
                error: None,
            })
            .collect(),
    )
}

fn labels_for_mode(mode: InputMode) -> (&'static str, &'static str) {
    match mode {
        InputMode::Pt => ("p (МПа)", "T (К)"),
        InputMode::Rhot => ("rho (кг/м^3)", "T (К)"),
        InputMode::Px => ("p (МПа)", "x"),
        InputMode::Ps => ("p (МПа)", "s (кДж/кгК)"),
        InputMode::Ph => ("p (МПа)", "h (кДж/кг)"),
    }
}

fn schedule_table_calculation(
    snapshot: crate::types::PersistentState,
    state_ctx: StateContext,
    app_ctx: AppContext,
    custom_name: UseStateHandle<String>,
    pending_timeout: Rc<RefCell<Option<gloo_timers::callback::Timeout>>>,
    request_seq: Rc<RefCell<u64>>,
    delay_ms: u32,
) {
    if let Some(timeout) = pending_timeout.borrow_mut().take() {
        timeout.cancel();
    }

    if snapshot.t_input.trim().is_empty() {
        let mut next = snapshot;
        next.t_res = Rc::new(Vec::new());
        state_ctx.set(next);
        return;
    }

    let timeout = gloo_timers::callback::Timeout::new(delay_ms, move || {
        let ticket = {
            let mut seq = request_seq.borrow_mut();
            *seq += 1;
            *seq
        };

        wasm_bindgen_futures::spawn_local(async move {
            let result = tauri_api::calculate_table(TableCalcRequest {
                mode: snapshot.t_mode,
                input: snapshot.t_input.clone(),
            })
            .await;

            if *request_seq.borrow() != ticket {
                return;
            }

            let mut next = (*state_ctx).clone();
            if next.t_mode != snapshot.t_mode || next.t_input != snapshot.t_input {
                return;
            }

            match result {
                Ok(rows) => {
                    next.t_res = Rc::new(rows);
                    if let Some(existing) = app_ctx.items.iter().find_map(|item| match item {
                        SavedItem::Table(table)
                            if table_matches_request(next.t_mode, &next.t_input, table) =>
                        {
                            Some(table)
                        }
                        _ => None,
                    }) {
                        custom_name.set(existing.name.clone());
                    }
                }
                Err(error) => {
                    next.t_res = Rc::new(vec![TableRowResult {
                        line_no: 0,
                        state: None,
                        error: Some(error),
                    }]);
                }
            }

            state_ctx.set(next);
        });
    });

    *pending_timeout.borrow_mut() = Some(timeout);
}

#[function_component(TableCalcTab)]
pub fn table_calc_tab() -> Html {
    let state_ctx = use_context::<StateContext>().expect("Контекст состояния не найден");
    let app_ctx = use_context::<AppContext>().expect("Контекст данных не найден");

    let custom_name = use_state(String::new);
    let save_status = use_state(|| Option::<String>::None);
    let notice = use_state(|| Option::<String>::None);
    let show_all_rows = use_state(|| false);
    let selected_row = use_state(|| Option::<usize>::None);
    let selected_col = use_state(|| Option::<usize>::None);
    let pending_timeout = use_mut_ref(|| Option::<gloo_timers::callback::Timeout>::None);
    let request_seq = use_mut_ref(|| 0_u64);

    let s = &*state_ctx;
    let (label_v1, label_v2) = labels_for_mode(s.t_mode);
    let gen_params = s.t_gen_params.get(&s.t_mode).cloned().unwrap_or_default();
    let valid_rows = valid_states(s.t_res.as_ref());
    let has_valid_results = !valid_rows.is_empty();
    let is_already_saved = app_ctx.items.iter().any(|item| match item {
        SavedItem::Table(table) => table_matches_request(s.t_mode, &s.t_input, table),
        SavedItem::Point(_) => false,
    });

    use_effect_with((s.t_mode, s.t_input.clone()), {
        let state_ctx = state_ctx.clone();
        let app_ctx = app_ctx.clone();
        let custom_name = custom_name.clone();
        let pending_timeout = pending_timeout.clone();
        let request_seq = request_seq.clone();
        move |_| {
            schedule_table_calculation(
                (*state_ctx).clone(),
                state_ctx.clone(),
                app_ctx.clone(),
                custom_name.clone(),
                pending_timeout.clone(),
                request_seq.clone(),
                180,
            );
            || ()
        }
    });

    let on_mode = {
        let state_ctx = state_ctx.clone();
        let save_status = save_status.clone();
        let notice = notice.clone();
        let show_all_rows = show_all_rows.clone();
        let selected_row = selected_row.clone();
        let selected_col = selected_col.clone();
        Callback::from(move |event: Event| {
            if let Some(select) = event.target_dyn_into::<HtmlSelectElement>() {
                let mut next = (*state_ctx).clone();
                next.t_mode = match select.value().as_str() {
                    "pt" => InputMode::Pt,
                    "ph" => InputMode::Ph,
                    "ps" => InputMode::Ps,
                    "px" => InputMode::Px,
                    "rhot" => InputMode::Rhot,
                    _ => InputMode::Pt,
                };
                state_ctx.set(next.clone());
                save_status.set(None);
                notice.set(None);
                show_all_rows.set(false);
                selected_row.set(None);
                selected_col.set(None);
            }
        })
    };

    let on_input = {
        let state_ctx = state_ctx.clone();
        let save_status = save_status.clone();
        let notice = notice.clone();
        let show_all_rows = show_all_rows.clone();
        let selected_row = selected_row.clone();
        let selected_col = selected_col.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(textarea) = event.target_dyn_into::<HtmlTextAreaElement>() {
                let mut next = (*state_ctx).clone();
                next.t_input = textarea.value();
                state_ctx.set(next.clone());
                save_status.set(None);
                notice.set(None);
                show_all_rows.set(false);
                selected_row.set(None);
                selected_col.set(None);
            }
        })
    };

    let on_precision_change = {
        let state_ctx = state_ctx.clone();
        Callback::from(move |event: Event| {
            if let Some(select) = event.target_dyn_into::<HtmlSelectElement>() {
                if let Ok(precision) = select.value().parse::<usize>() {
                    let mut next = (*state_ctx).clone();
                    next.t_precision = precision.min(10);
                    state_ctx.set(next);
                }
            }
        })
    };

    let on_gen_input = |field: &'static str| {
        let state_ctx = state_ctx.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
                let mut next = (*state_ctx).clone();
                let mut params = next.t_gen_params.get(&next.t_mode).cloned().unwrap_or_default();
                match field {
                    "v1_from" => params.v1_from = input.value(),
                    "v1_to" => params.v1_to = input.value(),
                    "v1_step" => params.v1_step = input.value(),
                    "v2_from" => params.v2_from = input.value(),
                    "v2_to" => params.v2_to = input.value(),
                    "v2_step" => params.v2_step = input.value(),
                    _ => {}
                }
                next.t_gen_params.insert(next.t_mode, params);
                state_ctx.set(next);
            }
        })
    };

    let on_generate = {
        let state_ctx = state_ctx.clone();
        let notice = notice.clone();
        let save_status = save_status.clone();
        let show_all_rows = show_all_rows.clone();
        let selected_row = selected_row.clone();
        let selected_col = selected_col.clone();
        Callback::from(move |_| {
            let mut next = (*state_ctx).clone();
            let params = next.t_gen_params.get(&next.t_mode).cloned().unwrap_or_default();

            let parse_range = |from: &str, to: &str, step: &str| -> Option<(f64, f64, f64, usize)> {
                let from = from.replace(',', ".").parse::<f64>().ok()?;
                let to = to.replace(',', ".").parse::<f64>().unwrap_or(from);
                let step = step.replace(',', ".").parse::<f64>().unwrap_or(1.0);

                let steps = if step == 0.0 || (to > from && step < 0.0) || (to < from && step > 0.0) {
                    0
                } else {
                    ((to - from) / step + 1e-9).max(0.0).floor() as usize
                };

                Some((from, to, step, steps))
            };

            if let (Some((from1, _, step1, steps1)), Some((from2, _, step2, steps2))) = (
                parse_range(&params.v1_from, &params.v1_to, &params.v1_step),
                parse_range(&params.v2_from, &params.v2_to, &params.v2_step),
            ) {
                let total_rows = (steps1 + 1).saturating_mul(steps2 + 1);
                if total_rows > MAX_GENERATED_ROWS {
                    notice.set(Some(format!(
                        "Превышен лимит генератора: запрошено {total_rows} строк, максимум {MAX_GENERATED_ROWS}"
                    )));
                    return;
                }

                let mut generated = String::new();
                for index1 in 0..=steps1 {
                    let value1 = from1 + (index1 as f64) * step1;
                    let value1 = (value1 * 1_000_000_000.0).round() / 1_000_000_000.0;
                    for index2 in 0..=steps2 {
                        let value2 = from2 + (index2 as f64) * step2;
                        let value2 = (value2 * 1_000_000_000.0).round() / 1_000_000_000.0;
                        generated.push_str(&format!("{value1};{value2}\n"));
                    }
                }

                if !next.t_input.is_empty() && !next.t_input.ends_with('\n') {
                    next.t_input.push('\n');
                }
                next.t_input.push_str(&generated);
                state_ctx.set(next);
                notice.set(None);
                save_status.set(None);
                show_all_rows.set(false);
                selected_row.set(None);
                selected_col.set(None);
            } else {
                notice.set(Some("Параметры генератора заполнены не полностью".to_string()));
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

    let on_load_file = {
        let state_ctx = state_ctx.clone();
        let notice = notice.clone();
        let save_status = save_status.clone();
        let show_all_rows = show_all_rows.clone();
        let selected_row = selected_row.clone();
        let selected_col = selected_col.clone();
        Callback::from(move |_| {
            let state_ctx = state_ctx.clone();
            let notice = notice.clone();
            let save_status = save_status.clone();
            let show_all_rows = show_all_rows.clone();
            let selected_row = selected_row.clone();
            let selected_col = selected_col.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match tauri_api::load_file_dialog().await {
                    Ok(content) => {
                        let mut next = (*state_ctx).clone();
                        next.t_input = content;
                        state_ctx.set(next.clone());
                        notice.set(None);
                        save_status.set(None);
                        show_all_rows.set(false);
                        selected_row.set(None);
                        selected_col.set(None);
                    }
                    Err(error) if error != "Отменено" => notice.set(Some(error)),
                    Err(_) => {}
                }
            });
        })
    };

    let on_export_csv = {
        let rows = s.t_res.clone();
        let precision = s.t_precision;
        let notice = notice.clone();
        Callback::from(move |_| {
            if rows.is_empty() {
                return;
            }

            let mut csv = String::from("строка;p (МПа);T (К);v (м3/кг);rho (кг/м3);h (кДж/кг);s (кДж/кгК);u (кДж/кг);cp (кДж/кгК);w (м/с);регион;ошибка\n");
            for row in rows.iter() {
                if let Some(state) = &row.state {
                    csv.push_str(&format!(
                        "{};{:.*};{:.*};{:.*};{:.*};{:.*};{:.*};{:.*};{:.*};{:.*};{};\n",
                        row.line_no,
                        precision,
                        state.p,
                        precision,
                        state.t,
                        precision,
                        state.v,
                        precision,
                        state.rho,
                        precision,
                        state.h,
                        precision,
                        state.s,
                        precision,
                        state.u,
                        precision,
                        state.cp,
                        precision,
                        state.w,
                        state.region
                    ));
                } else {
                    csv.push_str(&format!(
                        "{};;;;;;;;;;;{}\n",
                        row.line_no,
                        row.error.clone().unwrap_or_default()
                    ));
                }
            }

            let notice = notice.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Err(error) = tauri_api::save_file_dialog(tauri_api::SaveFileArgs {
                    content: csv,
                    default_name: Some("table_export.csv".to_string()),
                    filter_name: Some("CSV-документ".to_string()),
                    filter_ext: Some("csv".to_string()),
                })
                .await
                {
                    if error != "Отменено" {
                        notice.set(Some(error));
                    }
                }
            });
        })
    };

    let on_save = {
        let state_ctx = state_ctx.clone();
        let app_ctx = app_ctx.clone();
        let custom_name = custom_name.clone();
        let save_status = save_status.clone();
        Callback::from(move |_| {
            let valid_states = valid_states(state_ctx.t_res.as_ref());
            if valid_states.is_empty() {
                return;
            }

            let mut saved = (*app_ctx).clone();
            let table_name = if custom_name.trim().is_empty() {
                format!(
                    "Таблица {}",
                    saved
                        .items
                        .iter()
                        .filter(|item| matches!(item, SavedItem::Table(_)))
                        .count()
                        + 1
                )
            } else {
                (*custom_name).clone()
            };

            if let Some(SavedItem::Table(existing)) = saved.items.iter_mut().find(|item| match item {
                SavedItem::Table(table) => table_matches_request(state_ctx.t_mode, &state_ctx.t_input, table),
                SavedItem::Point(_) => false,
            }) {
                existing.name = table_name.clone();
                existing.states = valid_states;
                existing.orig_mode = state_ctx.t_mode;
                existing.orig_input = state_ctx.t_input.clone();
                save_status.set(Some("Обновлено".to_string()));
            } else {
                let id = saved.allocate_id();
                saved.items.push(SavedItem::Table(SavedTable {
                    id,
                    name: table_name.clone(),
                    states: valid_states,
                    orig_mode: state_ctx.t_mode,
                    orig_input: state_ctx.t_input.clone(),
                }));
                save_status.set(Some("Сохранено".to_string()));
            }

            custom_name.set(table_name);
            app_ctx.set(saved);
        })
    };

    let on_select_col = |column: usize| {
        let selected_col = selected_col.clone();
        let selected_row = selected_row.clone();
        Callback::from(move |_| {
            if *selected_col == Some(column) {
                selected_col.set(None);
            } else {
                selected_col.set(Some(column));
            }
            selected_row.set(None);
        })
    };

    let clear_selection = {
        let selected_row = selected_row.clone();
        let selected_col = selected_col.clone();
        Callback::from(move |_| {
            selected_row.set(None);
            selected_col.set(None);
        })
    };

    let visible_rows = if *show_all_rows {
        s.t_res.as_ref().clone()
    } else {
        s.t_res.iter().take(MAX_VISIBLE_ROWS).cloned().collect::<Vec<_>>()
    };

    html! {
        <div style="position: relative; height: 100%; min-height: 0; overflow: auto; display: flex; flex-direction: column;">
            <div class="table-calc-container fade-in" style="flex-grow: 1; overflow-y: auto; padding-bottom: 20px; display: flex; flex-direction: column;">
                <div class="card instruction-card table-top-strip">
                    <div class="table-top-strip-scroll">
                        <div class="table-top-strip-inner">
                            <h2 class="table-top-title">{"Табличный расчет"}</h2>
                            <div style="width: 1px; height: 24px; background: var(--border); margin: 0 4px;"></div>

                            <label class="table-top-label">{"Режим:"}</label>
                            <select class="styled-select" value={s.t_mode.as_str()} onchange={on_mode} style="min-width: 82px;">
                                <option value="pt">{"p-T"}</option>
                                <option value="rhot">{"rho-T"}</option>
                                <option value="px">{"p-x"}</option>
                                <option value="ps">{"p-s"}</option>
                                <option value="ph">{"p-h"}</option>
                            </select>

                            <label class="table-top-label">{"Точность:"}</label>
                            <select class="styled-select" value={s.t_precision.to_string()} onchange={on_precision_change} style="width: 66px;">
                                { for (0..=10).map(|precision| html! {
                                    <option value={precision.to_string()} selected={precision == s.t_precision}>{precision}</option>
                                }) }
                            </select>

                            <div style="width: 1px; height: 24px; background: var(--border); margin: 0 4px;"></div>
                            <button class="btn btn-outline btn-sm" onclick={on_load_file}>{"Открыть"}</button>
                            <button class="btn btn-success btn-sm" onclick={on_export_csv} disabled={s.t_res.is_empty()} style={if s.t_res.is_empty() { "opacity: 0.5; cursor: not-allowed;" } else { "" }}>{"Экспорт CSV"}</button>
                            <button class="btn btn-outline btn-sm" onclick={clear_selection}>{"Сброс выделения"}</button>

                            <div style="width: 1px; height: 24px; background: var(--border); margin: 0 4px;"></div>
                            <input type="text" class={classes!("styled-input", "table-top-name")} value={(*custom_name).clone()} oninput={on_name_input} placeholder="Имя сохраненной таблицы..." disabled={!has_valid_results} />
                            <button class={classes!("btn", "btn-sm", if is_already_saved { "btn-primary" } else { "btn-success" })} onclick={on_save} disabled={!has_valid_results} style={if !has_valid_results { "opacity: 0.5; cursor: not-allowed;" } else { "" }}>
                                { if is_already_saved { "Обновить" } else { "Сохранить" } }
                            </button>
                            { if let Some(status) = &*save_status {
                                html! { <span class="fade-in" style="color: #198754; font-size: 0.8rem; font-weight: 600; white-space: nowrap;">{status}</span> }
                            } else {
                                html! {}
                            }}
                        </div>
                    </div>
                </div>

                { if let Some(message) = &*notice {
                    html! { <div class="card fade-in" style="margin-bottom: 15px; border-color: #0d6efd; background-color: #e8f1ff; color: #0a3d91;">{ message }</div> }
                } else { html! {} } }

                <div class="card" style="margin-bottom: 15px; padding: 10px 15px; background: var(--hover-bg); display: flex; align-items: center; gap: 15px; flex-wrap: wrap;">
                    <div style="font-weight: 600; font-size: 0.95rem; white-space: nowrap; color: var(--primary);">{"Генератор"}</div>

                    <div style="display: flex; align-items: center; gap: 6px; flex-grow: 1; min-width: 250px;">
                        <span style="font-weight: 500; font-size: 0.85rem; color: var(--text-muted); width: 95px; text-align: right; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;" title={label_v1}>{ label_v1 }</span>
                        <input class="styled-input" placeholder="От" value={gen_params.v1_from.clone()} oninput={on_gen_input("v1_from")} style="flex-grow: 1; width: 10px; min-width: 40px; padding: 6px 8px; font-size: 0.85rem;" />
                        <input class="styled-input" placeholder="До" value={gen_params.v1_to.clone()} oninput={on_gen_input("v1_to")} style="flex-grow: 1; width: 10px; min-width: 40px; padding: 6px 8px; font-size: 0.85rem;" />
                        <input class="styled-input" placeholder="Шаг" value={gen_params.v1_step.clone()} oninput={on_gen_input("v1_step")} style="flex-grow: 1; width: 10px; min-width: 40px; padding: 6px 8px; font-size: 0.85rem;" />
                    </div>

                    <div style="width: 1px; height: 24px; background: var(--border);"></div>

                    <div style="display: flex; align-items: center; gap: 6px; flex-grow: 1; min-width: 250px;">
                        <span style="font-weight: 500; font-size: 0.85rem; color: var(--text-muted); width: 95px; text-align: right; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;" title={label_v2}>{ label_v2 }</span>
                        <input class="styled-input" placeholder="От" value={gen_params.v2_from.clone()} oninput={on_gen_input("v2_from")} style="flex-grow: 1; width: 10px; min-width: 40px; padding: 6px 8px; font-size: 0.85rem;" />
                        <input class="styled-input" placeholder="До" value={gen_params.v2_to.clone()} oninput={on_gen_input("v2_to")} style="flex-grow: 1; width: 10px; min-width: 40px; padding: 6px 8px; font-size: 0.85rem;" />
                        <input class="styled-input" placeholder="Шаг" value={gen_params.v2_step.clone()} oninput={on_gen_input("v2_step")} style="flex-grow: 1; width: 10px; min-width: 40px; padding: 6px 8px; font-size: 0.85rem;" />
                    </div>

                    <button class="btn btn-primary btn-sm" onclick={on_generate} style="white-space: nowrap; padding: 6px 12px; margin-left: auto;">{"Добавить строки"}</button>
                </div>

                <div class="split-view" style="flex-grow: 1;">
                    <div class="split-left" style="width: 24%;">
                        <textarea class="raw-data-area" oninput={on_input} value={s.t_input.clone()} placeholder="Вставьте исходные данные..."></textarea>
                    </div>
                    <div class="split-right" style="width: 76%; overflow-x: auto;">
                        <div class="table-container">
                            <table class="data-table" style="white-space: nowrap;">
                                <thead>
                                    <tr>
                                        <th class="table-index-head" style="width: 48px;">{"Строка"}</th>
                                        <th class={classes!("table-col-head", (*selected_col == Some(0)).then_some("table-selected-head"))} onclick={on_select_col(0)}>{"p (МПа)"}</th>
                                        <th class={classes!("table-col-head", (*selected_col == Some(1)).then_some("table-selected-head"))} onclick={on_select_col(1)}>{"T (K)"}</th>
                                        <th class={classes!("table-col-head", (*selected_col == Some(2)).then_some("table-selected-head"))} onclick={on_select_col(2)}>{"v (m^3/kg)"}</th>
                                        <th class={classes!("table-col-head", (*selected_col == Some(3)).then_some("table-selected-head"))} onclick={on_select_col(3)}>{"rho (kg/m^3)"}</th>
                                        <th class={classes!("table-col-head", (*selected_col == Some(4)).then_some("table-selected-head"))} onclick={on_select_col(4)}>{"h (kJ/kg)"}</th>
                                        <th class={classes!("table-col-head", (*selected_col == Some(5)).then_some("table-selected-head"))} onclick={on_select_col(5)}>{"s (kJ/kgK)"}</th>
                                        <th class={classes!("table-col-head", (*selected_col == Some(6)).then_some("table-selected-head"))} onclick={on_select_col(6)}>{"u (kJ/kg)"}</th>
                                        <th class={classes!("table-col-head", (*selected_col == Some(7)).then_some("table-selected-head"))} onclick={on_select_col(7)}>{"cp (kJ/kgK)"}</th>
                                        <th class={classes!("table-col-head", (*selected_col == Some(8)).then_some("table-selected-head"))} onclick={on_select_col(8)}>{"w (m/s)"}</th>
                                        <th class={classes!("table-col-head", (*selected_col == Some(9)).then_some("table-selected-head"))} onclick={on_select_col(9)}>{"Регион"}</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    { for visible_rows.iter().map(|row| {
                                        let row_no = row.line_no;
                                        let row_selected = *selected_row == Some(row_no);
                                        let on_row_select = {
                                            let selected_row = selected_row.clone();
                                            let selected_col = selected_col.clone();
                                            Callback::from(move |_| {
                                                if *selected_row == Some(row_no) {
                                                    selected_row.set(None);
                                                } else {
                                                    selected_row.set(Some(row_no));
                                                }
                                                selected_col.set(None);
                                            })
                                        };
                                        if let Some(state) = &row.state {
                                            html! {
                                                <tr>
                                                    <td class={classes!("table-index-cell", row_selected.then_some("table-selected-cell"))} onclick={on_row_select.clone()}>{row_no}</td>
                                                    <td class={classes!((row_selected || *selected_col == Some(0)).then_some("table-selected-cell"))}>{format_value(state.p, s.t_precision)}</td>
                                                    <td class={classes!((row_selected || *selected_col == Some(1)).then_some("table-selected-cell"))}>{format_value(state.t, s.t_precision)}</td>
                                                    <td class={classes!((row_selected || *selected_col == Some(2)).then_some("table-selected-cell"))}>{format_value(state.v, s.t_precision)}</td>
                                                    <td class={classes!((row_selected || *selected_col == Some(3)).then_some("table-selected-cell"))}>{format_value(state.rho, s.t_precision)}</td>
                                                    <td class={classes!((row_selected || *selected_col == Some(4)).then_some("table-selected-cell"))}>{format_value(state.h, s.t_precision)}</td>
                                                    <td class={classes!((row_selected || *selected_col == Some(5)).then_some("table-selected-cell"))}>{format_value(state.s, s.t_precision)}</td>
                                                    <td class={classes!((row_selected || *selected_col == Some(6)).then_some("table-selected-cell"))}>{format_value(state.u, s.t_precision)}</td>
                                                    <td class={classes!((row_selected || *selected_col == Some(7)).then_some("table-selected-cell"))}>{format_value(state.cp, s.t_precision)}</td>
                                                    <td class={classes!((row_selected || *selected_col == Some(8)).then_some("table-selected-cell"))}>{format_value(state.w, s.t_precision)}</td>
                                                    <td class={classes!((row_selected || *selected_col == Some(9)).then_some("table-selected-cell"))}>{state.region.clone()}</td>
                                                </tr>
                                            }
                                        } else {
                                            html! {
                                                <tr class="table-error-row">
                                                    <td class={classes!("table-index-cell", row_selected.then_some("table-selected-cell"))} onclick={on_row_select}>{row_no}</td>
                                                    <td colspan="10" class="table-error-message">
                                                        {format!("Ошибка: {}", row.error.clone().unwrap_or_else(|| "Неизвестная ошибка".to_string()))}
                                                    </td>
                                                </tr>
                                            }
                                        }
                                    }) }
                                </tbody>
                            </table>
                        </div>
                        { if s.t_res.len() > MAX_VISIBLE_ROWS && !*show_all_rows {
                            let show_all_rows = show_all_rows.clone();
                            html! {
                                <div style="padding: 12px 0;">
                                    <button class="btn btn-outline btn-sm" onclick={Callback::from(move |_| show_all_rows.set(true))}>
                                        {format!("Показать все строки ({})", s.t_res.len())}
                                    </button>
                                </div>
                            }
                        } else { html! {} } }
                    </div>
                </div>
            </div>

            <div style={format!("position: absolute; top: 0; left: 0; bottom: 0; width: 320px; background: var(--card-bg); border-right: 1px solid var(--border); box-shadow: 4px 0 15px rgba(0,0,0,0.15); transform: translateX({}); transition: transform 0.3s cubic-bezier(0.4, 0.0, 0.2, 1); display: flex; flex-direction: column; z-index: 50;", if s.right_sidebar_open { "0" } else { "-120%" })}>
                <div style="padding: 15px; border-bottom: 1px solid var(--border); background: var(--hover-bg);">
                    <h3 style="margin: 0; font-size: 1.1rem;">{"Сохранённые таблицы"}</h3>
                </div>
                <div style="padding: 15px; overflow-y: auto; flex-grow: 1; display: flex; flex-direction: column; gap: 8px;">
                    { if !app_ctx.items.iter().any(|item| matches!(item, SavedItem::Table(_))) { html! { <div style="color: var(--text-muted); font-size: 0.9rem; text-align: center; margin-top: 20px;">{"Нет сохранённых таблиц"}</div> } } else { html! {} } }

                    { for app_ctx.items.iter().filter_map(|item| if let SavedItem::Table(table) = item { Some(table) } else { None }).map(|table| {
                        let on_load = {
                            let state_ctx = state_ctx.clone();
                            let custom_name = custom_name.clone();
                            let selected_row = selected_row.clone();
                            let selected_col = selected_col.clone();
                            let table = table.clone();
                            Callback::from(move |_| {
                                let mut next = (*state_ctx).clone();
                                next.t_mode = table.orig_mode;
                                next.t_input = table.orig_input.clone();
                                next.t_res = rows_from_states(&table.states);
                                state_ctx.set(next);
                                custom_name.set(table.name.clone());
                                selected_row.set(None);
                                selected_col.set(None);
                            })
                        };
                        let on_delete = {
                            let app_ctx = app_ctx.clone();
                            let state_ctx = state_ctx.clone();
                            let table_id = table.id;
                            Callback::from(move |event: MouseEvent| {
                                event.stop_propagation();
                                let mut saved = (*app_ctx).clone();
                                saved.items.retain(|item| item.id() != table_id);
                                app_ctx.set(saved);

                                let mut next = (*state_ctx).clone();
                                next.plot_selected.remove(&table_id);
                                state_ctx.set(next);
                            })
                        };

                        html! {
                            <div style="display: flex; justify-content: space-between; align-items: center; padding: 10px; background: var(--bg-color); border: 1px solid var(--border); border-radius: 6px; cursor: pointer;" onclick={on_load}>
                                <span style="font-weight: 500; font-size: 0.95rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">{ &table.name }</span>
                                <button class="btn btn-sm" style="background: transparent; border: none; color: #dc3545; padding: 5px; font-size: 1.1rem; cursor: pointer;" onclick={on_delete} title="Удалить">{"X"}</button>
                            </div>
                        }
                    }) }
                </div>
            </div>
        </div>
    }
}
