//! Вкладка построения диаграмм (canvas).

use crate::plot::{ChartOptions, PlotSeries, draw_diagram, project_state, render_diagram_to_svg};
use crate::tauri_api;
use crate::types::{AppContext, SavedItem, StateContext};
use gloo_timers::callback::Timeout;
use if97_app_api::{AxisVar, DomeRequest, PlotPoint};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::{JsCast, closure::Closure};
use web_sys::{
    HtmlCanvasElement, HtmlElement, HtmlInputElement, HtmlSelectElement, MouseEvent, WheelEvent,
};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
/// Параметры вкладки графиков.
pub struct PlotsProps {
    /// Флаг активности вкладки (используется для управления отрисовкой).
    pub active: bool,
}

#[derive(Clone, Copy, PartialEq)]
struct PlotRanges {
    p: (f64, f64),
    t: (f64, f64),
    v: (f64, f64),
    rho: (f64, f64),
    h: (f64, f64),
    s: (f64, f64),
    u: (f64, f64),
    cp: (f64, f64),
    w: (f64, f64),
    x: (f64, f64),
}

impl Default for PlotRanges {
    fn default() -> Self {
        Self {
            p: (0.001, 100.0),
            t: (273.15, 1000.0),
            v: (0.001, 2.0),
            rho: (1.0, 1200.0),
            h: (0.0, 4000.0),
            s: (0.0, 10.0),
            u: (0.0, 3500.0),
            cp: (0.0, 50.0),
            w: (0.0, 2000.0),
            x: (0.0, 1.0),
        }
    }
}

impl PlotRanges {
    fn get(&self, var: AxisVar) -> (f64, f64) {
        match var {
            AxisVar::P => self.p,
            AxisVar::T => self.t,
            AxisVar::V => self.v,
            AxisVar::Rho => self.rho,
            AxisVar::H => self.h,
            AxisVar::S => self.s,
            AxisVar::U => self.u,
            AxisVar::Cp => self.cp,
            AxisVar::W => self.w,
            AxisVar::X => self.x,
        }
    }

    fn set_min(&mut self, var: AxisVar, value: f64) {
        match var {
            AxisVar::P => self.p.0 = value,
            AxisVar::T => self.t.0 = value,
            AxisVar::V => self.v.0 = value,
            AxisVar::Rho => self.rho.0 = value,
            AxisVar::H => self.h.0 = value,
            AxisVar::S => self.s.0 = value,
            AxisVar::U => self.u.0 = value,
            AxisVar::Cp => self.cp.0 = value,
            AxisVar::W => self.w.0 = value,
            AxisVar::X => self.x.0 = value,
        }
    }

    fn set_max(&mut self, var: AxisVar, value: f64) {
        match var {
            AxisVar::P => self.p.1 = value,
            AxisVar::T => self.t.1 = value,
            AxisVar::V => self.v.1 = value,
            AxisVar::Rho => self.rho.1 = value,
            AxisVar::H => self.h.1 = value,
            AxisVar::S => self.s.1 = value,
            AxisVar::U => self.u.1 = value,
            AxisVar::Cp => self.cp.1 = value,
            AxisVar::W => self.w.1 = value,
            AxisVar::X => self.x.1 = value,
        }
    }
}

#[function_component(PlotsTab)]
/// Вкладка графиков: выбор диаграммы, управление масштабом и отрисовка данных.
pub fn plots_tab(props: &PlotsProps) -> Html {
    let app_ctx = use_context::<AppContext>().expect("Контекст данных не найден");
    let state_ctx = use_context::<StateContext>().expect("Контекст состояния не найден");

    let canvas_ref = use_node_ref();
    let x_axis = use_state(|| AxisVar::T);
    let y_axis = use_state(|| AxisVar::P);
    let x_log = use_state(|| false);
    let y_log = use_state(|| false);
    let show_dome = use_state(|| true);
    let ranges = use_state(PlotRanges::default);
    let resize_tick = use_state(|| 0u32);
    let is_dragging = use_state(|| false);
    let last_mouse = use_state(|| (0.0, 0.0));
    let dome_points = use_mut_ref(|| Rc::<Vec<PlotPoint>>::new(Vec::new()));
    let dome_rev = use_state(|| 0u32);
    let dome_req_id = use_mut_ref(|| 0u64);
    let dome_cache = use_mut_ref(|| HashMap::<(AxisVar, AxisVar), Rc<Vec<PlotPoint>>>::new());
    let pending_ranges = use_mut_ref(|| Option::<PlotRanges>::None);
    let pending_ranges_update = use_mut_ref(|| Option::<Timeout>::None);

    let x_var = *x_axis;
    let y_var = *y_axis;
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

    use_effect({
        let resize_tick = resize_tick.clone();
        move || {
            // Обработчик ресайза окна нужен, чтобы подгонять внутренний буфер canvas и
            // перерисовывать график без искажений.
            let mut cleanup: Option<(
                web_sys::Window,
                Closure<dyn FnMut(web_sys::Event)>,
                Rc<RefCell<Option<Timeout>>>,
            )> = None;

            if let Some(window) = web_sys::window() {
                let pending = Rc::new(RefCell::new(None::<Timeout>));
                let pending_for_cb = pending.clone();
                let resize_tick_for_cb = resize_tick.clone();
                let closure = Closure::<dyn FnMut(web_sys::Event)>::wrap(Box::new(move |_| {
                    if pending_for_cb.borrow().is_some() {
                        return;
                    }

                    let pending_for_timer = pending_for_cb.clone();
                    let resize_tick_for_timer = resize_tick_for_cb.clone();
                    let timeout = Timeout::new(80, move || {
                        pending_for_timer.borrow_mut().take();
                        resize_tick_for_timer.set(*resize_tick_for_timer + 1);
                    });
                    *pending_for_cb.borrow_mut() = Some(timeout);
                }));

                let _ = window
                    .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref());

                cleanup = Some((window, closure, pending));
            }

            move || {
                if let Some((window, closure, pending)) = cleanup {
                    pending.borrow_mut().take();
                    let _ = window.remove_event_listener_with_callback(
                        "resize",
                        closure.as_ref().unchecked_ref(),
                    );
                    drop(closure);
                }
            }
        }
    });

    let selected_ids = state_ctx.plot_selected.clone();
    let series_list = use_memo(
        (app_ctx.clone(), x_var, y_var, selected_ids.clone()),
        |(saved, x_var, y_var, selected_ids)| {
            saved
                .items
                .iter()
                .filter(|item| selected_ids.contains(&item.id()))
                .map(|item| {
                    let points = match item {
                        SavedItem::Point(point) => {
                            vec![project_state(*x_var, *y_var, &point.state)]
                        }
                        SavedItem::Table(table) => table
                            .states
                            .iter()
                            .map(|state| project_state(*x_var, *y_var, state))
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

    use_effect_with((x_var, y_var, *show_dome), {
        let dome_points = dome_points.clone();
        let dome_rev = dome_rev.clone();
        let dome_req_id = dome_req_id.clone();
        let dome_cache = dome_cache.clone();
        move |(x_var, y_var, show_dome)| {
            let req_id = {
                let mut id = dome_req_id.borrow_mut();
                *id += 1;
                *id
            };

            if !*show_dome {
                *dome_points.borrow_mut() = Rc::new(Vec::new());
            } else {
                let cache_key = (*x_var, *y_var);
                if let Some(cached) = dome_cache.borrow().get(&cache_key) {
                    *dome_points.borrow_mut() = cached.clone();
                } else {
                    // Не показываем "старый" купол, пока не приехали точки для новой диаграммы.
                    // Иначе при переключении диаграмм/осей на короткое время виден мусор.
                    *dome_points.borrow_mut() = Rc::new(Vec::new());

                    let dome_points = dome_points.clone();
                    let dome_rev = dome_rev.clone();
                    let dome_req_id = dome_req_id.clone();
                    let dome_cache = dome_cache.clone();
                    let x_var = *x_var;
                    let y_var = *y_var;
                    wasm_bindgen_futures::spawn_local(async move {
                        match tauri_api::calculate_dome(DomeRequest { x_var, y_var }).await {
                            Ok(points) => {
                                let points = Rc::new(points);
                                dome_cache
                                    .borrow_mut()
                                    .insert((x_var, y_var), points.clone());
                                if *dome_req_id.borrow() == req_id {
                                    *dome_points.borrow_mut() = points;
                                    dome_rev.set(*dome_rev + 1);
                                }
                            }
                            Err(error) => tracing::error!("ошибка расчета купола: {error}"),
                        }
                    });
                }
            }

            || ()
        }
    });

    use_effect_with(
        (
            canvas_ref.clone(),
            props.active,
            *resize_tick,
            x_var,
            y_var,
            *x_log,
            *y_log,
            *show_dome,
            *ranges,
            *dome_rev,
            series_list.clone(),
        ),
        {
            let dome_points = dome_points.clone();
            move |(
                canvas_ref,
                is_active,
                _resize_tick,
                x_var,
                y_var,
                x_log,
                y_log,
                show_dome,
                ranges,
                _dome_rev,
                series_list,
            )| {
                if *is_active {
                    if let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() {
                        // Подгоняем внутренний буфер canvas под текущий размер элемента, чтобы:
                        // 1) не было искажений/letterbox при CSS-скейле,
                        // 2) график оставался резким на Retina.
                        let css_w = canvas.client_width() as f64;
                        let css_h = canvas.client_height() as f64;
                        if css_w > 0.0 && css_h > 0.0 {
                            let dpr = web_sys::window()
                                .map(|w| w.device_pixel_ratio())
                                .unwrap_or(1.0)
                                .max(1.0)
                                .min(2.0);
                            let next_w = (css_w * dpr).round() as u32;
                            let next_h = (css_h * dpr).round() as u32;
                            if canvas.width() != next_w {
                                canvas.set_width(next_w);
                            }
                            if canvas.height() != next_h {
                                canvas.set_height(next_h);
                            }
                        }

                        let opts = ChartOptions {
                            show_dome: *show_dome,
                            x_range: ranges.get(*x_var),
                            y_range: ranges.get(*y_var),
                            x_log: *x_log,
                            y_log: *y_log,
                        };
                        let dome_points = dome_points.borrow();
                        if let Err(error) = draw_diagram(
                            &canvas,
                            &opts,
                            dome_points.as_slice(),
                            series_list.as_ref(),
                        ) {
                            tracing::error!("ошибка отрисовки графика: {error}");
                        }
                    }
                }

                || ()
            }
        },
    );

    fn clamp_positive_range(mut min: f64, mut max: f64) -> (f64, f64) {
        if min > max {
            std::mem::swap(&mut min, &mut max);
        }
        let eps = 1e-12;
        if !min.is_finite() || min <= 0.0 {
            min = eps;
        }
        if !max.is_finite() || max <= min {
            max = min * 10.0;
        }
        (min, max)
    }

    let toggle_dome = {
        let show_dome = show_dome.clone();
        Callback::from(move |_| show_dome.set(!*show_dome))
    };

    let on_x_axis_change = {
        let x_axis = x_axis.clone();
        let ranges = ranges.clone();
        let x_log = x_log.clone();
        Callback::from(move |event: Event| {
            if let Some(select) = event.target_dyn_into::<HtmlSelectElement>() {
                if let Ok(index) = select.value().parse::<usize>() {
                    let var = AxisVar::from_index(index);
                    x_axis.set(var);
                    if *x_log {
                        let mut next = *ranges;
                        let (min, max) = next.get(var);
                        let (min, max) = clamp_positive_range(min, max);
                        next.set_min(var, min);
                        next.set_max(var, max);
                        ranges.set(next);
                    }
                }
            }
        })
    };

    let on_y_axis_change = {
        let y_axis = y_axis.clone();
        let ranges = ranges.clone();
        let y_log = y_log.clone();
        Callback::from(move |event: Event| {
            if let Some(select) = event.target_dyn_into::<HtmlSelectElement>() {
                if let Ok(index) = select.value().parse::<usize>() {
                    let var = AxisVar::from_index(index);
                    y_axis.set(var);
                    if *y_log {
                        let mut next = *ranges;
                        let (min, max) = next.get(var);
                        let (min, max) = clamp_positive_range(min, max);
                        next.set_min(var, min);
                        next.set_max(var, max);
                        ranges.set(next);
                    }
                }
            }
        })
    };

    let toggle_x_log = {
        let x_log = x_log.clone();
        let ranges = ranges.clone();
        Callback::from(move |_| {
            let next_flag = !*x_log;
            x_log.set(next_flag);
            if next_flag {
                let mut next = *ranges;
                let (min, max) = next.get(x_var);
                let (min, max) = clamp_positive_range(min, max);
                next.set_min(x_var, min);
                next.set_max(x_var, max);
                ranges.set(next);
            }
        })
    };

    let toggle_y_log = {
        let y_log = y_log.clone();
        let ranges = ranges.clone();
        Callback::from(move |_| {
            let next_flag = !*y_log;
            y_log.set(next_flag);
            if next_flag {
                let mut next = *ranges;
                let (min, max) = next.get(y_var);
                let (min, max) = clamp_positive_range(min, max);
                next.set_min(y_var, min);
                next.set_max(y_var, max);
                ranges.set(next);
            }
        })
    };

    let on_x_min_change = {
        let ranges = ranges.clone();
        let x_log = x_log.clone();
        Callback::from(move |event: Event| {
            if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
                if let Ok(value) = input.value().parse::<f64>() {
                    let mut next = *ranges;
                    next.set_min(x_var, value);
                    if *x_log {
                        let (min, max) = next.get(x_var);
                        let (min, max) = clamp_positive_range(min, max);
                        next.set_min(x_var, min);
                        next.set_max(x_var, max);
                    }
                    ranges.set(next);
                }
            }
        })
    };

    let on_x_max_change = {
        let ranges = ranges.clone();
        let x_log = x_log.clone();
        Callback::from(move |event: Event| {
            if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
                if let Ok(value) = input.value().parse::<f64>() {
                    let mut next = *ranges;
                    next.set_max(x_var, value);
                    if *x_log {
                        let (min, max) = next.get(x_var);
                        let (min, max) = clamp_positive_range(min, max);
                        next.set_min(x_var, min);
                        next.set_max(x_var, max);
                    }
                    ranges.set(next);
                }
            }
        })
    };

    let on_y_min_change = {
        let ranges = ranges.clone();
        let y_log = y_log.clone();
        Callback::from(move |event: Event| {
            if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
                if let Ok(value) = input.value().parse::<f64>() {
                    let mut next = *ranges;
                    next.set_min(y_var, value);
                    if *y_log {
                        let (min, max) = next.get(y_var);
                        let (min, max) = clamp_positive_range(min, max);
                        next.set_min(y_var, min);
                        next.set_max(y_var, max);
                    }
                    ranges.set(next);
                }
            }
        })
    };

    let on_y_max_change = {
        let ranges = ranges.clone();
        let y_log = y_log.clone();
        Callback::from(move |event: Event| {
            if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
                if let Ok(value) = input.value().parse::<f64>() {
                    let mut next = *ranges;
                    next.set_max(y_var, value);
                    if *y_log {
                        let (min, max) = next.get(y_var);
                        let (min, max) = clamp_positive_range(min, max);
                        next.set_min(y_var, min);
                        next.set_max(y_var, max);
                    }
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
        let x_var = x_var;
        let y_var = y_var;
        let x_is_log = *x_log;
        let y_is_log = *y_log;
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

                let x_range = next.get(x_var);
                let y_range = next.get(y_var);

                if x_is_log {
                    let (min, max) = clamp_positive_range(x_range.0, x_range.1);
                    let (min, max) = (min.log10(), max.log10());
                    let mouse = min + fx * (max - min);
                    let new_span = (max - min) * zoom;
                    let new_min = mouse - fx * new_span;
                    let new_max = mouse + (1.0 - fx) * new_span;
                    next.set_min(x_var, 10_f64.powf(new_min));
                    next.set_max(x_var, 10_f64.powf(new_max));
                } else {
                    let (min, max) = (x_range.0.min(x_range.1), x_range.0.max(x_range.1));
                    let mouse = min + fx * (max - min);
                    let new_span = (max - min) * zoom;
                    next.set_min(x_var, mouse - fx * new_span);
                    next.set_max(x_var, mouse + (1.0 - fx) * new_span);
                }

                if y_is_log {
                    let (min, max) = clamp_positive_range(y_range.0, y_range.1);
                    let (min, max) = (min.log10(), max.log10());
                    let mouse = max - fy * (max - min);
                    let new_span = (max - min) * zoom;
                    let new_min = mouse - (1.0 - fy) * new_span;
                    let new_max = mouse + fy * new_span;
                    next.set_min(y_var, 10_f64.powf(new_min));
                    next.set_max(y_var, 10_f64.powf(new_max));
                } else {
                    let (min, max) = (y_range.0.min(y_range.1), y_range.0.max(y_range.1));
                    let mouse = max - fy * (max - min);
                    let new_span = (max - min) * zoom;
                    next.set_min(y_var, mouse - (1.0 - fy) * new_span);
                    next.set_max(y_var, mouse + fy * new_span);
                }

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
        let x_var = x_var;
        let y_var = y_var;
        let x_is_log = *x_log;
        let y_is_log = *y_log;
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
                let x_range = next.get(x_var);
                let y_range = next.get(y_var);

                if x_is_log {
                    let (min, max) = clamp_positive_range(x_range.0, x_range.1);
                    let (min, max) = (min.log10(), max.log10());
                    let shift = dx * (max - min) / width;
                    let new_min = min - shift;
                    let new_max = max - shift;
                    next.set_min(x_var, 10_f64.powf(new_min));
                    next.set_max(x_var, 10_f64.powf(new_max));
                } else {
                    let (min, max) = (x_range.0.min(x_range.1), x_range.0.max(x_range.1));
                    let shift = dx * (max - min) / width;
                    next.set_min(x_var, min - shift);
                    next.set_max(x_var, max - shift);
                }

                if y_is_log {
                    let (min, max) = clamp_positive_range(y_range.0, y_range.1);
                    let (min, max) = (min.log10(), max.log10());
                    let shift = dy * (max - min) / height;
                    let new_min = min + shift;
                    let new_max = max + shift;
                    next.set_min(y_var, 10_f64.powf(new_min));
                    next.set_max(y_var, 10_f64.powf(new_max));
                } else {
                    let (min, max) = (y_range.0.min(y_range.1), y_range.0.max(y_range.1));
                    let shift = dy * (max - min) / height;
                    next.set_min(y_var, min + shift);
                    next.set_max(y_var, max + shift);
                }

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

    let on_save_svg = {
        let show_dome = show_dome.clone();
        let ranges = ranges.clone();
        let x_log = x_log.clone();
        let y_log = y_log.clone();
        let dome_points = dome_points.clone();
        let series_list = series_list.clone();
        Callback::from(move |_| {
            let opts = ChartOptions {
                show_dome: *show_dome,
                x_range: ranges.get(x_var),
                y_range: ranges.get(y_var),
                x_log: *x_log,
                y_log: *y_log,
            };
            let dome_points = dome_points.borrow();
            match render_diagram_to_svg(
                &opts,
                dome_points.as_slice(),
                series_list.as_ref(),
                1920,
                1080,
            ) {
                Ok(svg) => {
                    wasm_bindgen_futures::spawn_local(async move {
                        let _ = tauri_api::save_file_dialog(tauri_api::SaveFileArgs {
                            content: svg,
                            default_name: Some("if97_plot.svg".to_string()),
                            filter_name: Some("SVG".to_string()),
                            filter_ext: Some("svg".to_string()),
                        })
                        .await;
                    });
                }
                Err(error) => tracing::error!("ошибка генерации SVG: {error}"),
            }
        })
    };

    let axis_controls = |axis: &'static str,
                         var: AxisVar,
                         is_log: bool,
                         range: (f64, f64),
                         on_axis_change: Callback<Event>,
                         on_toggle_log: Callback<MouseEvent>,
                         on_min_change: Callback<Event>,
                         on_max_change: Callback<Event>| {
        html! {
            <div style="display: flex; align-items: center; gap: 8px; background: var(--hover-bg); padding: 6px 10px; border-radius: 8px; border: 1px solid var(--border);">
                <span style="font-weight: 700; font-size: 0.85rem; width: 14px;">{ axis }</span>
                <select class="styled-select" value={var.to_index().to_string()} onchange={on_axis_change} style="min-width: 160px; max-width: 240px;">
                    { for AxisVar::ALL.iter().copied().map(|candidate| {
                        let idx = candidate.to_index().to_string();
                        html! {
                            <option value={idx} selected={candidate == var}>
                                { candidate.label() }
                            </option>
                        }
                    }) }
                </select>
                <label class="toggle-label" style="font-size: 0.85rem; display: flex; align-items: center; gap: 6px;">
                    <input type="checkbox" checked={is_log} onclick={on_toggle_log} />
                    { "log" }
                </label>
                <input type="number" step="any" class="styled-input" style="width: 90px; padding: 4px; font-size: 0.85rem;" value={range.0.to_string()} onchange={on_min_change} />
                <span style="color: var(--text-muted);">{"-"}</span>
                <input type="number" step="any" class="styled-input" style="width: 90px; padding: 4px; font-size: 0.85rem;" value={range.1.to_string()} onchange={on_max_change} />
            </div>
        }
    };

    html! {
        <div class="charts-container fade-in" style="display: flex; flex-direction: column; height: 100%; min-height: 0; overflow: hidden; position: relative;">
            <div class="top-toolbar" style="display: flex; flex-wrap: wrap; gap: 15px; padding: 10px 15px; background: var(--card-bg); border-bottom: 1px solid var(--border); align-items: center; flex-shrink: 0; z-index: 5;">
                { axis_controls(
                    "X",
                    x_var,
                    *x_log,
                    x_range,
                    on_x_axis_change.clone(),
                    toggle_x_log.clone(),
                    on_x_min_change.clone(),
                    on_x_max_change.clone(),
                ) }
                { axis_controls(
                    "Y",
                    y_var,
                    *y_log,
                    y_range,
                    on_y_axis_change.clone(),
                    toggle_y_log.clone(),
                    on_y_min_change.clone(),
                    on_y_max_change.clone(),
                ) }

                <label class="toggle-label" style="font-size: 0.85rem; padding: 6px 10px; background: var(--hover-bg); border-radius: 8px; border: 1px solid var(--border);">
                    <input type="checkbox" checked={*show_dome} onclick={toggle_dome} />
                    { "Купол" }
                </label>

                <button class="btn btn-outline btn-sm" onclick={on_reset_scales} title="Сбросить диапазоны графика">{"Сброс"}</button>
                <button class="btn btn-success btn-sm" onclick={on_save_png}>{"Сохранить PNG"}</button>
                <button class="btn btn-success btn-sm" onclick={on_save_svg}>{"Сохранить SVG"}</button>
            </div>

            <div style="flex-grow: 1; position: relative; overflow: hidden; background: var(--bg-color);">
                <canvas id="plot-area" ref={canvas_ref} width="1800" height="1200" style="width: 100%; height: 100%; display: block; cursor: crosshair;" onwheel={on_wheel} onmousedown={on_mouse_down} onmouseup={on_mouse_up} onmousemove={on_mouse_move} onmouseleave={on_mouse_leave}></canvas>

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
