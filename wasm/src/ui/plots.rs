/* File: src/ui/plots.rs */
use yew::prelude::*;
use std::collections::HashSet;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, HtmlElement, HtmlInputElement, HtmlSelectElement, MouseEvent, WheelEvent};
use crate::types::{AppContext, StateContext, ChartType, SavedItem};
use crate::plot::{draw_diagram, ChartOptions, PlotSeries};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    pub async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[derive(Properties, PartialEq)]
pub struct PlotsProps { pub active: bool }

#[derive(Clone, Copy, PartialEq)]
pub enum AxisVar { P, T, H, S, V }

impl AxisVar {
    fn name(&self) -> &'static str {
        match self {
            AxisVar::P => "p (МПа)", AxisVar::T => "T (K)", AxisVar::H => "h (кДж/кг)",
            AxisVar::S => "s (кДж/кгK)", AxisVar::V => "v (м³/кг)",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct PlotRanges {
    p: (f64, f64), t: (f64, f64), h: (f64, f64), s: (f64, f64), v: (f64, f64),
}

impl Default for PlotRanges {
    fn default() -> Self {
        Self {
            p: (0.001, 100.0), t: (273.15, 1000.0), h: (0.0, 4000.0), s: (0.0, 10.0), v: (0.001, 2.0),
        }
    }
}

impl PlotRanges {
    fn get(&self, var: AxisVar) -> (f64, f64) {
        match var { AxisVar::P => self.p, AxisVar::T => self.t, AxisVar::H => self.h, AxisVar::S => self.s, AxisVar::V => self.v }
    }
    fn set_min(&mut self, var: AxisVar, val: f64) {
        match var { AxisVar::P => self.p.0 = val, AxisVar::T => self.t.0 = val, AxisVar::H => self.h.0 = val, AxisVar::S => self.s.0 = val, AxisVar::V => self.v.0 = val }
    }
    fn set_max(&mut self, var: AxisVar, val: f64) {
        match var { AxisVar::P => self.p.1 = val, AxisVar::T => self.t.1 = val, AxisVar::H => self.h.1 = val, AxisVar::S => self.s.1 = val, AxisVar::V => self.v.1 = val }
    }
}

fn get_axes(ct: ChartType, swap: bool) -> (AxisVar, AxisVar) {
    let (mut x, mut y) = match ct {
        ChartType::Ts => (AxisVar::S, AxisVar::T), ChartType::Hs => (AxisVar::S, AxisVar::H),
        ChartType::Ph => (AxisVar::H, AxisVar::P), ChartType::Tv => (AxisVar::V, AxisVar::T),
        ChartType::Pv => (AxisVar::V, AxisVar::P), ChartType::Pt => (AxisVar::T, AxisVar::P),
        ChartType::Ps => (AxisVar::S, AxisVar::P), ChartType::Th => (AxisVar::H, AxisVar::T),
    };
    if swap { std::mem::swap(&mut x, &mut y); }
    (x, y)
}

#[function_component(PlotsTab)]
pub fn plots_tab(props: &PlotsProps) -> Html {
    let global_ctx = use_context::<AppContext>().expect("AppContext not found");
    let state_ctx = use_context::<StateContext>().expect("StateContext not found");
    let s = &*state_ctx;

    let canvas_ref = use_node_ref();
    let chart_type = use_state(|| ChartType::Pt); // Теперь по умолчанию p-T
    let swap_axes = use_state(|| false);
    let draw_lines = use_state(|| false);
    let show_dome = use_state(|| true);
    let ranges = use_state(|| PlotRanges::default());
    let is_dragging = use_state(|| false);
    let last_mouse = use_state(|| (0.0, 0.0));

    let (x_var, y_var) = get_axes(*chart_type, *swap_axes);
    let x_range = ranges.get(x_var);
    let y_range = ranges.get(y_var);

    let selected_items = s.plot_selected.clone();

    // МЕМОИЗАЦИЯ ДАННЫХ: Выполняется только если изменились исходные данные, тип графика или выборка
    let series_list_memo = use_memo(
        (global_ctx.clone(), *chart_type, *swap_axes, selected_items.clone()),
        |(ctx, ct, swap, selected)| {
            let mut series_list = Vec::new();
            for item in ctx.iter() {
                let (name, states) = match item {
                    SavedItem::Point(p) => (p.name.clone(), vec![p.state.clone()]),
                    SavedItem::Table(t) => (t.name.clone(), t.states.clone()),
                };
                if selected.contains(&name) {
                    let points: Vec<(f64, f64)> = states.into_iter().map(|state| {
                        let (mut x, mut y) = match ct {
                            ChartType::Ts => (state.s.inner(), state.t.inner()), ChartType::Hs => (state.s.inner(), state.h.inner()),
                            ChartType::Ph => (state.h.inner(), state.p.inner()), ChartType::Tv => (state.v.inner(), state.t.inner()),
                            ChartType::Pv => (state.v.inner(), state.p.inner()), ChartType::Pt => (state.t.inner(), state.p.inner()),
                            ChartType::Ps => (state.s.inner(), state.p.inner()), ChartType::Th => (state.h.inner(), state.t.inner()),
                        };
                        if *swap { std::mem::swap(&mut x, &mut y); }
                        (x, y)
                    }).collect();
                    series_list.push(PlotSeries { name, points });
                }
            }
            series_list
        }
    );

    // ОТРИСОВКА: Выполняется при каждом изменении масштаба (ranges), но не пересчитывает данные с нуля
    use_effect_with((
                        canvas_ref.clone(), props.active,
                        *chart_type, *swap_axes, *draw_lines, *show_dome, *ranges, series_list_memo.clone()
                    ), |(canvas_ref, is_active, ct, swap, dl, dome, rng, series_list)| {
        if *is_active {
            if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                let (x_v, y_v) = get_axes(*ct, *swap);
                let opts = ChartOptions {
                    chart_type: *ct, swap_axes: *swap, draw_lines: *dl, show_dome: *dome,
                    x_range: rng.get(x_v), y_range: rng.get(y_v)
                };
                let canvas_id = "plot-area".to_string();

                // ИСПРАВЛЕНИЕ: Двойное разыменование достает Vec из-под Rc
                let series_to_draw = (**series_list).clone();

                wasm_bindgen_futures::spawn_local(async move { let _ = draw_diagram(&canvas_id, &opts, series_to_draw); });
            }
        }
        || ()
    });

    let on_chart_type_change = {
        let chart_type = chart_type.clone();
        Callback::from(move |e: Event| {
            if let Some(select) = e.target_dyn_into::<HtmlSelectElement>() {
                let ct = match select.value().as_str() {
                    "pt" => ChartType::Pt,
                    "pv" => ChartType::Pv,
                    "ph" => ChartType::Ph,
                    "ps" => ChartType::Ps,
                    "tv" => ChartType::Tv,
                    "ts" => ChartType::Ts,
                    "th" => ChartType::Th,
                    "hs" => ChartType::Hs,
                    _ => ChartType::Pt
                };
                chart_type.set(ct);
            }
        })
    };

    let toggle_swap = { let s = swap_axes.clone(); Callback::from(move |_| { s.set(!*s); }) };
    let toggle_lines = { let s = draw_lines.clone(); Callback::from(move |_| s.set(!*s)) };
    let toggle_dome = { let s = show_dome.clone(); Callback::from(move |_| s.set(!*s)) };

    let on_min_change = |var: AxisVar| {
        let ranges = ranges.clone();
        Callback::from(move |e: Event| {
            if let Some(i) = e.target_dyn_into::<HtmlInputElement>() {
                if let Ok(v) = i.value().parse() { let mut r = *ranges; r.set_min(var, v); ranges.set(r); }
            }
        })
    };

    let on_max_change = |var: AxisVar| {
        let ranges = ranges.clone();
        Callback::from(move |e: Event| {
            if let Some(i) = e.target_dyn_into::<HtmlInputElement>() {
                if let Ok(v) = i.value().parse() { let mut r = *ranges; r.set_max(var, v); ranges.set(r); }
            }
        })
    };

    let on_reset_scales = { let ranges = ranges.clone(); Callback::from(move |_| ranges.set(PlotRanges::default())) };

    let on_wheel = {
        let ranges = ranges.clone(); let ct = *chart_type; let swap = *swap_axes;
        Callback::from(move |e: WheelEvent| {
            e.prevent_default();
            if let Some(el) = e.target_dyn_into::<HtmlElement>() {
                let w = el.client_width() as f64; let h = el.client_height() as f64;
                if w == 0.0 || h == 0.0 { return; }
                let fx = e.offset_x() as f64 / w; let fy = e.offset_y() as f64 / h;
                let zoom_factor = if e.delta_y() > 0.0 { 1.1 } else { 0.9 };
                let mut r = *ranges;
                let (x_v, y_v) = get_axes(ct, swap);
                let xr = r.get(x_v); let yr = r.get(y_v);
                let mouse_data_x = xr.0 + fx * (xr.1 - xr.0);
                let mouse_data_y = yr.1 - fy * (yr.1 - yr.0);
                let new_span_x = (xr.1 - xr.0) * zoom_factor;
                let new_span_y = (yr.1 - yr.0) * zoom_factor;
                r.set_min(x_v, mouse_data_x - fx * new_span_x);
                r.set_max(x_v, mouse_data_x + (1.0 - fx) * new_span_x);
                r.set_min(y_v, mouse_data_y - (1.0 - fy) * new_span_y);
                r.set_max(y_v, mouse_data_y + fy * new_span_y);
                ranges.set(r);
            }
        })
    };

    let on_mouse_down = { let is_drag = is_dragging.clone(); let last_m = last_mouse.clone(); Callback::from(move |e: MouseEvent| { is_drag.set(true); last_m.set((e.client_x() as f64, e.client_y() as f64)); }) };
    let on_mouse_up = { let is_drag = is_dragging.clone(); Callback::from(move |_| { is_drag.set(false); }) };
    let on_mouse_leave = { let is_drag = is_dragging.clone(); Callback::from(move |_| { is_drag.set(false); }) };

    let on_mouse_move = {
        let is_drag = is_dragging.clone(); let last_m = last_mouse.clone();
        let ranges = ranges.clone(); let ct = *chart_type; let swap = *swap_axes;
        Callback::from(move |e: MouseEvent| {
            if *is_drag {
                if let Some(el) = e.target_dyn_into::<HtmlElement>() {
                    let w = el.client_width() as f64; let h = el.client_height() as f64;
                    if w == 0.0 || h == 0.0 { return; }
                    let dx = e.client_x() as f64 - last_m.0; let dy = e.client_y() as f64 - last_m.1;
                    last_m.set((e.client_x() as f64, e.client_y() as f64));
                    let mut r = *ranges;
                    let (x_v, y_v) = get_axes(ct, swap);
                    let xr = r.get(x_v); let yr = r.get(y_v);
                    let x_shift = dx * (xr.1 - xr.0) / w; let y_shift = dy * (yr.1 - yr.0) / h;
                    r.set_min(x_v, xr.0 - x_shift); r.set_max(x_v, xr.1 - x_shift);
                    r.set_min(y_v, yr.0 + y_shift); r.set_max(y_v, yr.1 + y_shift);
                    ranges.set(r);
                }
            }
        })
    };

    let on_save_png = {
        let canvas_ref = canvas_ref.clone();
        Callback::from(move |_| {
            if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                if let Ok(data_url) = canvas.to_data_url_with_type("image/png") {
                    wasm_bindgen_futures::spawn_local(async move {
                        let b64_data = data_url.split(',').nth(1).unwrap_or("");
                        #[derive(serde::Serialize)] struct SavePlotArgs { b64: String }
                        if let Ok(args) = serde_wasm_bindgen::to_value(&SavePlotArgs { b64: b64_data.to_string() }) {
                            let _ = invoke("save_plot_dialog", args).await;
                        }
                    });
                }
            }
        })
    };

    let scale_input = |var: AxisVar, range: (f64, f64)| {
        html! {
            <div style="display: flex; align-items: center; gap: 5px; background: var(--hover-bg); padding: 4px 8px; border-radius: 6px; border: 1px solid var(--border);">
                <span style="font-weight: 600; font-size: 0.85rem; width: 85px;">{ var.name() }</span>
                <input type="number" step="any" class="styled-input" style="width: 90px; padding: 4px; font-size: 0.85rem;" value={range.0.to_string()} onchange={on_min_change(var)} />
                <span style="color: var(--text-muted);">{"-"}</span>
                <input type="number" step="any" class="styled-input" style="width: 90px; padding: 4px; font-size: 0.85rem;" value={range.1.to_string()} onchange={on_max_change(var)} />
            </div>
        }
    };

    html! {
        <div class="charts-container fade-in" style="display: flex; flex-direction: column; height: calc(100vh - 40px); overflow: hidden; position: relative;">

            <div class="top-toolbar" style="display: flex; flex-wrap: wrap; gap: 15px; padding: 10px 15px; background: var(--card-bg); border-bottom: 1px solid var(--border); align-items: center; flex-shrink: 0; z-index: 5;">
                <select class="styled-select" onchange={on_chart_type_change} style="padding: 6px 10px;">
                    <option value="pt" selected={*chart_type == ChartType::Pt}>{ "p-T Диаграмма" }</option>
                    <option value="pv" selected={*chart_type == ChartType::Pv}>{ "p-v Диаграмма" }</option>
                    <option value="ps" selected={*chart_type == ChartType::Ps}>{ "p-s Диаграмма" }</option>
                    <option value="ph" selected={*chart_type == ChartType::Ph}>{ "p-h Диаграмма" }</option>
                    <option value="tv" selected={*chart_type == ChartType::Tv}>{ "T-v Диаграмма" }</option>
                    <option value="ts" selected={*chart_type == ChartType::Ts}>{ "T-s Диаграмма" }</option>
                    <option value="th" selected={*chart_type == ChartType::Th}>{ "T-h Диаграмма" }</option>
                    <option value="hs" selected={*chart_type == ChartType::Hs}>{ "h-s Диаграмма" }</option>
                </select>

                <div style="display: flex; gap: 10px; border-right: 1px solid var(--border); padding-right: 15px;">
                    <label class="toggle-label" style="font-size: 0.85rem;"><input type="checkbox" checked={*swap_axes} onclick={toggle_swap} /> { "Оси 🔄" }</label>
                    <label class="toggle-label" style="font-size: 0.85rem;"><input type="checkbox" checked={*show_dome} onclick={toggle_dome} /> { "Купол" }</label>
                    <label class="toggle-label" style="font-size: 0.85rem;"><input type="checkbox" checked={*draw_lines} onclick={toggle_lines} /> { "Линии" }</label>
                </div>

                { scale_input(x_var, x_range) }
                { scale_input(y_var, y_range) }

                <button class="btn btn-outline btn-sm" onclick={on_reset_scales} title="Сбросить все масштабы">{"🔄"}</button>
                <button class="btn btn-success btn-sm" onclick={on_save_png}>{"💾 PNG"}</button>
            </div>

            <div style="flex-grow: 1; position: relative; overflow: hidden; background: var(--bg-color);">
                <canvas id="plot-area" ref={canvas_ref} width="2400" height="1600" style="width: 100%; height: 100%; object-fit: contain; cursor: crosshair;" onwheel={on_wheel} onmousedown={on_mouse_down} onmouseup={on_mouse_up} onmousemove={on_mouse_move} onmouseleave={on_mouse_leave}></canvas>

                <div style={format!("position: absolute; top: 0; left: 0; bottom: 0; width: 320px; background: var(--card-bg); border-right: 1px solid var(--border); box-shadow: 4px 0 15px rgba(0,0,0,0.15); transform: translateX({}); transition: transform 0.3s cubic-bezier(0.4, 0.0, 0.2, 1); display: flex; flex-direction: column; z-index: 50;", if s.right_sidebar_open { "0" } else { "-120%" })}>
                    <div style="padding: 15px; border-bottom: 1px solid var(--border); background: var(--hover-bg);">
                        <h3 style="margin: 0; font-size: 1.1rem;">{"Управление данными"}</h3>
                    </div>

                    <div style="padding: 15px; overflow-y: auto; flex-grow: 1; display: flex; flex-direction: column; gap: 8px;">
                        { if global_ctx.is_empty() { html! { <div style="color: var(--text-muted); font-size: 0.9rem; text-align: center; margin-top: 20px;">{"Нет данных для отрисовки"}</div> } } else { html! {} } }

                        { for global_ctx.iter().map(|item| {
                            let name = match item { SavedItem::Point(p) => p.name.clone(), SavedItem::Table(t) => t.name.clone() };
                            let is_checked = s.plot_selected.contains(&name);

                            let on_toggle = {
                                let state_ctx = state_ctx.clone(); let name = name.clone();
                                Callback::from(move |_| {
                                    let mut new_s = (*state_ctx).clone();
                                    if new_s.plot_selected.contains(&name) { new_s.plot_selected.remove(&name); }
                                    else { new_s.plot_selected.insert(name.clone()); }
                                    state_ctx.set(new_s);
                                })
                            };

                            html! {
                                <label class="toggle-label" style="font-size: 0.95rem; padding: 6px; background: var(--hover-bg); border-radius: 4px; border: 1px solid transparent; display: flex; align-items: center; gap: 8px;">
                                    <input type="checkbox" checked={is_checked} onclick={on_toggle} />
                                    <span>{ match item { SavedItem::Point(_) => "📍 ", SavedItem::Table(_) => "📋 " } }{ name }</span>
                                </label>
                            }
                        }) }
                    </div>
                </div>
            </div>
        </div>
    }
}