use crate::plot::{draw_diagram, project_state, ChartOptions, PlotSeries};
use crate::tauri_api;
use crate::types::{AppContext, ChartType, SavedItem, StateContext};
use gloo_timers::callback::Timeout;
use if97_app_api::{DomeRequest, PlotPoint};
use std::rc::Rc;
use web_sys::{HtmlCanvasElement, HtmlElement, HtmlInputElement, HtmlSelectElement, MouseEvent, WheelEvent};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct PlotsProps {
    pub active: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum AxisVar {
    P,
    T,
    H,
    S,
    V,
}

impl AxisVar {
    fn name(&self) -> &'static str {
        match self {
            Self::P => "p (MPa)",
            Self::T => "T (K)",
            Self::H => "h (kJ/kg)",
            Self::S => "s (kJ/kgK)",
            Self::V => "v (m^3/kg)",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct PlotRanges {
    p: (f64, f64),
    t: (f64, f64),
    h: (f64, f64),
    s: (f64, f64),
    v: (f64, f64),
}

impl Default for PlotRanges {
    fn default() -> Self {
        Self {
            p: (0.001, 100.0),
            t: (273.15, 1000.0),
            h: (0.0, 4000.0),
            s: (0.0, 10.0),
            v: (0.001, 2.0),
        }
    }
}

impl PlotRanges {
    fn get(&self, var: AxisVar) -> (f64, f64) {
        match var {
            AxisVar::P => self.p,
            AxisVar::T => self.t,
            AxisVar::H => self.h,
            AxisVar::S => self.s,
            AxisVar::V => self.v,
        }
    }

    fn set_min(&mut self, var: AxisVar, value: f64) {
        match var {
            AxisVar::P => self.p.0 = value,
            AxisVar::T => self.t.0 = value,
            AxisVar::H => self.h.0 = value,
            AxisVar::S => self.s.0 = value,
            AxisVar::V => self.v.0 = value,
        }
    }

    fn set_max(&mut self, var: AxisVar, value: f64) {
        match var {
            AxisVar::P => self.p.1 = value,
            AxisVar::T => self.t.1 = value,
            AxisVar::H => self.h.1 = value,
            AxisVar::S => self.s.1 = value,
            AxisVar::V => self.v.1 = value,
        }
    }
}

fn get_axes(chart_type: ChartType, swap_axes: bool) -> (AxisVar, AxisVar) {
    let (mut x_var, mut y_var) = match chart_type {
        ChartType::Ts => (AxisVar::S, AxisVar::T),
        ChartType::Hs => (AxisVar::S, AxisVar::H),
        ChartType::Ph => (AxisVar::H, AxisVar::P),
        ChartType::Tv => (AxisVar::V, AxisVar::T),
        ChartType::Pv => (AxisVar::V, AxisVar::P),
        ChartType::Pt => (AxisVar::T, AxisVar::P),
        ChartType::Ps => (AxisVar::S, AxisVar::P),
        ChartType::Th => (AxisVar::H, AxisVar::T),
    };

    if swap_axes {
        std::mem::swap(&mut x_var, &mut y_var);
    }

    (x_var, y_var)
}

#[function_component(PlotsTab)]
pub fn plots_tab(props: &PlotsProps) -> Html {
    let app_ctx = use_context::<AppContext>().expect("Контекст данных не найден");
    let state_ctx = use_context::<StateContext>().expect("Контекст состояния не найден");

    let canvas_ref = use_node_ref();
    let chart_type = use_state(|| ChartType::Pt);
    let swap_axes = use_state(|| false);
    let show_dome = use_state(|| true);
    let ranges = use_state(PlotRanges::default);
    let is_dragging = use_state(|| false);
    let last_mouse = use_state(|| (0.0, 0.0));
    let dome_points = use_state(Vec::<PlotPoint>::new);
    let pending_ranges = use_mut_ref(|| Option::<PlotRanges>::None);
    let pending_ranges_update = use_mut_ref(|| Option::<Timeout>::None);

    let (x_var, y_var) = get_axes(*chart_type, *swap_axes);
    let x_range = ranges.get(x_var);
    let y_range = ranges.get(y_var);
    let schedule_ranges_update = Rc::new({
        let ranges = ranges.clone();
        let pending_ranges = pending_ranges.clone();
        let pending_ranges_update = pending_ranges_update.clone();
        move |next_ranges: PlotRanges| {
            *pending_ranges.borrow_mut() = Some(next_ranges);
            if pending_ranges_update.borrow().is_some() {
                return;
            }

            let ranges = ranges.clone();
            let pending_ranges = pending_ranges.clone();
            let pending_ranges_update = pending_ranges_update.clone();
            let pending_ranges_update_for_timer = pending_ranges_update.clone();
            let timeout = Timeout::new(12, move || {
                pending_ranges_update_for_timer.borrow_mut().take();
                if let Some(next_ranges) = pending_ranges.borrow_mut().take() {
                    ranges.set(next_ranges);
                }
            });
            *pending_ranges_update.borrow_mut() = Some(timeout);
        }
    });

    let selected_ids = state_ctx.plot_selected.clone();
    let series_list = use_memo(
        (app_ctx.clone(), *chart_type, *swap_axes, selected_ids.clone()),
        |(saved, chart_type, swap_axes, selected_ids)| {
            saved.items
                .iter()
                .filter(|item| selected_ids.contains(&item.id()))
                .map(|item| {
                    let points = match item {
                        SavedItem::Point(point) => vec![project_state(*chart_type, *swap_axes, &point.state)],
                        SavedItem::Table(table) => table
                            .states
                            .iter()
                            .map(|state| project_state(*chart_type, *swap_axes, state))
                            .collect(),
                    };

                    PlotSeries {
                        name: item.name().to_string(),
                        points,
                    }
                })
                .collect::<Vec<_>>()
        },
    );

    use_effect_with((*chart_type, *swap_axes, *show_dome), {
        let dome_points = dome_points.clone();
        move |(chart_type, swap_axes, show_dome)| {
            if !*show_dome {
                dome_points.set(Vec::new());
            } else {
                let dome_points = dome_points.clone();
                let chart_type = *chart_type;
                let swap_axes = *swap_axes;
                wasm_bindgen_futures::spawn_local(async move {
                    match tauri_api::calculate_dome(DomeRequest {
                        chart_type,
                        swap_axes,
                    })
                    .await
                    {
                        Ok(points) => dome_points.set(points),
                        Err(error) => tracing::error!("ошибка расчета купола: {error}"),
                    }
                });
            }

            || ()
        }
    });

    use_effect_with(
        (
            canvas_ref.clone(),
            props.active,
            *chart_type,
            *swap_axes,
            *show_dome,
            *ranges,
            dome_points.clone(),
            series_list.clone(),
        ),
        |(
            canvas_ref,
            is_active,
            chart_type,
            swap_axes,
            show_dome,
            ranges,
            dome_points,
            series_list,
        )| {
            if *is_active && canvas_ref.cast::<HtmlCanvasElement>().is_some() {
                let canvas_id = "plot-area".to_string();
                let (x_var, y_var) = get_axes(*chart_type, *swap_axes);
                let opts = ChartOptions {
                    show_dome: *show_dome,
                    x_range: ranges.get(x_var),
                    y_range: ranges.get(y_var),
                };
                if let Err(error) = draw_diagram(&canvas_id, &opts, dome_points.as_ref(), series_list.as_ref()) {
                    tracing::error!("ошибка отрисовки графика: {error}");
                }
            }

            || ()
        },
    );

    let on_chart_type_change = {
        let chart_type = chart_type.clone();
        Callback::from(move |event: Event| {
            if let Some(select) = event.target_dyn_into::<HtmlSelectElement>() {
                chart_type.set(match select.value().as_str() {
                    "pt" => ChartType::Pt,
                    "pv" => ChartType::Pv,
                    "ph" => ChartType::Ph,
                    "ps" => ChartType::Ps,
                    "tv" => ChartType::Tv,
                    "ts" => ChartType::Ts,
                    "th" => ChartType::Th,
                    "hs" => ChartType::Hs,
                    _ => ChartType::Pt,
                });
            }
        })
    };

    let toggle_swap = {
        let swap_axes = swap_axes.clone();
        Callback::from(move |_| swap_axes.set(!*swap_axes))
    };
    let toggle_dome = {
        let show_dome = show_dome.clone();
        Callback::from(move |_| show_dome.set(!*show_dome))
    };

    let on_min_change = |var: AxisVar| {
        let ranges = ranges.clone();
        Callback::from(move |event: Event| {
            if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
                if let Ok(value) = input.value().parse::<f64>() {
                    let mut next = *ranges;
                    next.set_min(var, value);
                    ranges.set(next);
                }
            }
        })
    };

    let on_max_change = |var: AxisVar| {
        let ranges = ranges.clone();
        Callback::from(move |event: Event| {
            if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
                if let Ok(value) = input.value().parse::<f64>() {
                    let mut next = *ranges;
                    next.set_max(var, value);
                    ranges.set(next);
                }
            }
        })
    };

    let on_reset_scales = {
        let ranges = ranges.clone();
        let pending_ranges = pending_ranges.clone();
        let pending_ranges_update = pending_ranges_update.clone();
        Callback::from(move |_| {
            if let Some(timeout) = pending_ranges_update.borrow_mut().take() {
                timeout.cancel();
            }
            pending_ranges.borrow_mut().take();
            ranges.set(PlotRanges::default());
        })
    };

    let on_wheel = {
        let ranges = ranges.clone();
        let schedule_ranges_update = schedule_ranges_update.clone();
        let chart_type = *chart_type;
        let swap_axes = *swap_axes;
        Callback::from(move |event: WheelEvent| {
            event.prevent_default();
            if let Some(element) = event.target_dyn_into::<HtmlElement>() {
                let width = element.client_width() as f64;
                let height = element.client_height() as f64;
                if width == 0.0 || height == 0.0 {
                    return;
                }

                let fx = event.offset_x() as f64 / width;
                let fy = event.offset_y() as f64 / height;
                let zoom = if event.delta_y() > 0.0 { 1.1 } else { 0.9 };
                let mut next = *ranges;
                let (x_var, y_var) = get_axes(chart_type, swap_axes);
                let x_range = next.get(x_var);
                let y_range = next.get(y_var);
                let mouse_x = x_range.0 + fx * (x_range.1 - x_range.0);
                let mouse_y = y_range.1 - fy * (y_range.1 - y_range.0);
                let new_span_x = (x_range.1 - x_range.0) * zoom;
                let new_span_y = (y_range.1 - y_range.0) * zoom;

                next.set_min(x_var, mouse_x - fx * new_span_x);
                next.set_max(x_var, mouse_x + (1.0 - fx) * new_span_x);
                next.set_min(y_var, mouse_y - (1.0 - fy) * new_span_y);
                next.set_max(y_var, mouse_y + fy * new_span_y);
                schedule_ranges_update(next);
            }
        })
    };

    let on_mouse_down = {
        let is_dragging = is_dragging.clone();
        let last_mouse = last_mouse.clone();
        Callback::from(move |event: MouseEvent| {
            is_dragging.set(true);
            last_mouse.set((event.client_x() as f64, event.client_y() as f64));
        })
    };

    let on_mouse_up = {
        let is_dragging = is_dragging.clone();
        Callback::from(move |_| is_dragging.set(false))
    };

    let on_mouse_leave = {
        let is_dragging = is_dragging.clone();
        Callback::from(move |_| is_dragging.set(false))
    };

    let on_mouse_move = {
        let is_dragging = is_dragging.clone();
        let last_mouse = last_mouse.clone();
        let ranges = ranges.clone();
        let schedule_ranges_update = schedule_ranges_update.clone();
        let chart_type = *chart_type;
        let swap_axes = *swap_axes;
        Callback::from(move |event: MouseEvent| {
            if !*is_dragging {
                return;
            }

            if let Some(element) = event.target_dyn_into::<HtmlElement>() {
                let width = element.client_width() as f64;
                let height = element.client_height() as f64;
                if width == 0.0 || height == 0.0 {
                    return;
                }

                let dx = event.client_x() as f64 - last_mouse.0;
                let dy = event.client_y() as f64 - last_mouse.1;
                last_mouse.set((event.client_x() as f64, event.client_y() as f64));

                let mut next = *ranges;
                let (x_var, y_var) = get_axes(chart_type, swap_axes);
                let x_range = next.get(x_var);
                let y_range = next.get(y_var);
                let x_shift = dx * (x_range.1 - x_range.0) / width;
                let y_shift = dy * (y_range.1 - y_range.0) / height;

                next.set_min(x_var, x_range.0 - x_shift);
                next.set_max(x_var, x_range.1 - x_shift);
                next.set_min(y_var, y_range.0 + y_shift);
                next.set_max(y_var, y_range.1 + y_shift);
                schedule_ranges_update(next);
            }
        })
    };

    let on_save_png = {
        let canvas_ref = canvas_ref.clone();
        Callback::from(move |_| {
            if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                if let Ok(data_url) = canvas.to_data_url_with_type("image/png") {
                    let encoded = data_url.split(',').nth(1).unwrap_or("").to_string();
                    wasm_bindgen_futures::spawn_local(async move {
                        let _ = tauri_api::save_plot_dialog(encoded).await;
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
        <div class="charts-container fade-in" style="display: flex; flex-direction: column; height: 100%; min-height: 0; overflow: hidden; position: relative;">
            <div class="top-toolbar" style="display: flex; flex-wrap: wrap; gap: 15px; padding: 10px 15px; background: var(--card-bg); border-bottom: 1px solid var(--border); align-items: center; flex-shrink: 0; z-index: 5;">
                <select class="styled-select" onchange={on_chart_type_change} style="padding: 6px 10px;">
                    <option value="pt" selected={*chart_type == ChartType::Pt}>{ "p-T диаграмма" }</option>
                    <option value="pv" selected={*chart_type == ChartType::Pv}>{ "p-v диаграмма" }</option>
                    <option value="ps" selected={*chart_type == ChartType::Ps}>{ "p-s диаграмма" }</option>
                    <option value="ph" selected={*chart_type == ChartType::Ph}>{ "p-h диаграмма" }</option>
                    <option value="tv" selected={*chart_type == ChartType::Tv}>{ "T-v диаграмма" }</option>
                    <option value="ts" selected={*chart_type == ChartType::Ts}>{ "T-s диаграмма" }</option>
                    <option value="th" selected={*chart_type == ChartType::Th}>{ "T-h диаграмма" }</option>
                    <option value="hs" selected={*chart_type == ChartType::Hs}>{ "h-s диаграмма" }</option>
                </select>

                <div style="display: flex; gap: 10px; border-right: 1px solid var(--border); padding-right: 15px;">
                    <label class="toggle-label" style="font-size: 0.85rem;"><input type="checkbox" checked={*swap_axes} onclick={toggle_swap} /> { "Оси местами" }</label>
                    <label class="toggle-label" style="font-size: 0.85rem;"><input type="checkbox" checked={*show_dome} onclick={toggle_dome} /> { "Купол" }</label>
                </div>

                { scale_input(x_var, x_range) }
                { scale_input(y_var, y_range) }

                <button class="btn btn-outline btn-sm" onclick={on_reset_scales} title="Сбросить диапазоны графика">{"Сброс"}</button>
                <button class="btn btn-success btn-sm" onclick={on_save_png}>{"Сохранить PNG"}</button>
            </div>

            <div style="flex-grow: 1; position: relative; overflow: hidden; background: var(--bg-color);">
                <canvas id="plot-area" ref={canvas_ref} width="1800" height="1200" style="width: 100%; height: 100%; object-fit: contain; cursor: crosshair;" onwheel={on_wheel} onmousedown={on_mouse_down} onmouseup={on_mouse_up} onmousemove={on_mouse_move} onmouseleave={on_mouse_leave}></canvas>

                <div style={format!("position: absolute; top: 0; left: 0; bottom: 0; width: 320px; background: var(--card-bg); border-right: 1px solid var(--border); box-shadow: 4px 0 15px rgba(0,0,0,0.15); transform: translateX({}); transition: transform 0.3s cubic-bezier(0.4, 0.0, 0.2, 1); display: flex; flex-direction: column; z-index: 50;", if state_ctx.right_sidebar_open { "0" } else { "-120%" })}>
                    <div style="padding: 15px; border-bottom: 1px solid var(--border); background: var(--hover-bg);">
                        <h3 style="margin: 0; font-size: 1.1rem;">{"Данные графика"}</h3>
                    </div>

                    <div style="padding: 15px; overflow-y: auto; flex-grow: 1; display: flex; flex-direction: column; gap: 8px;">
                        { if app_ctx.items.is_empty() { html! { <div style="color: var(--text-muted); font-size: 0.9rem; text-align: center; margin-top: 20px;">{"Нет сохраненных данных для отрисовки"}</div> } } else { html! {} } }

                        { for app_ctx.items.iter().map(|item| {
                            let item_id = item.id();
                            let is_checked = state_ctx.plot_selected.contains(&item_id);
                            let label = format!(
                                "{} {}",
                                match item {
                                    SavedItem::Point(_) => "Точка",
                                    SavedItem::Table(_) => "Таблица",
                                },
                                item.name()
                            );

                            let on_toggle = {
                                let state_ctx = state_ctx.clone();
                                Callback::from(move |_| {
                                    let mut next = (*state_ctx).clone();
                                    if next.plot_selected.contains(&item_id) {
                                        next.plot_selected.remove(&item_id);
                                    } else {
                                        next.plot_selected.insert(item_id);
                                    }
                                    state_ctx.set(next);
                                })
                            };

                            html! {
                                <label class="toggle-label" style="font-size: 0.95rem; padding: 6px; background: var(--hover-bg); border-radius: 4px; border: 1px solid transparent; display: flex; align-items: center; gap: 8px;">
                                    <input type="checkbox" checked={is_checked} onclick={on_toggle} />
                                    <span>{ label }</span>
                                </label>
                            }
                        }) }
                    </div>
                </div>
            </div>
        </div>
    }
}
