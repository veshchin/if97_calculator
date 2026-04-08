use crate::state::AppState;
use if97_app_api::DiagramKind;
use if97_core::{saturation, WaterState};
use plotters::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

fn get_palette_color(idx: usize) -> RGBColor {
    let palette = [
        RGBColor(41, 98, 255),
        RGBColor(220, 53, 69),
        RGBColor(25, 135, 84),
        RGBColor(255, 140, 0),
        RGBColor(111, 66, 193),
        RGBColor(13, 202, 240),
        RGBColor(108, 117, 125),
    ];
    palette[idx % palette.len()]
}

fn project_state(kind: DiagramKind, swap_axes: bool, state: &WaterState) -> (f64, f64) {
    let (mut x, mut y) = match kind {
        DiagramKind::Ts => (state.s.inner(), state.t.inner()),
        DiagramKind::Hs => (state.s.inner(), state.h.inner()),
        DiagramKind::Ph => (state.h.inner(), state.p.inner()),
        DiagramKind::Tv => (state.v.inner(), state.t.inner()),
        DiagramKind::Pv => (state.v.inner(), state.p.inner()),
        DiagramKind::Pt => (state.t.inner(), state.p.inner()),
        DiagramKind::Ps => (state.s.inner(), state.p.inner()),
        DiagramKind::Th => (state.h.inner(), state.t.inner()),
    };

    if swap_axes {
        std::mem::swap(&mut x, &mut y);
    }

    (x, y)
}

#[derive(Clone, Copy)]
enum AxisVar {
    P,
    T,
    H,
    S,
    V,
}

fn axis_vars(kind: DiagramKind, swap_axes: bool) -> (AxisVar, AxisVar) {
    let (mut x, mut y) = match kind {
        DiagramKind::Ts => (AxisVar::S, AxisVar::T),
        DiagramKind::Hs => (AxisVar::S, AxisVar::H),
        DiagramKind::Ph => (AxisVar::H, AxisVar::P),
        DiagramKind::Tv => (AxisVar::V, AxisVar::T),
        DiagramKind::Pv => (AxisVar::V, AxisVar::P),
        DiagramKind::Pt => (AxisVar::T, AxisVar::P),
        DiagramKind::Ps => (AxisVar::S, AxisVar::P),
        DiagramKind::Th => (AxisVar::H, AxisVar::T),
    };

    if swap_axes {
        std::mem::swap(&mut x, &mut y);
    }

    (x, y)
}

fn metric_value(var: AxisVar, value: f64) -> f64 {
    match var {
        AxisVar::V => value.max(1e-12).ln(),
        _ => value,
    }
}

fn axis_labels(kind: DiagramKind, swap_axes: bool) -> (&'static str, &'static str) {
    let (x, y) = match kind {
        DiagramKind::Ts => ("Энтропия s, кДж/(кг·К)", "Температура T, К"),
        DiagramKind::Hs => ("Энтропия s, кДж/(кг·К)", "Энтальпия h, кДж/кг"),
        DiagramKind::Ph => ("Энтальпия h, кДж/кг", "Давление p, МПа"),
        DiagramKind::Tv => ("Удельный объем v, м3/кг", "Температура T, К"),
        DiagramKind::Pv => ("Удельный объем v, м3/кг", "Давление p, МПа"),
        DiagramKind::Pt => ("Температура T, К", "Давление p, МПа"),
        DiagramKind::Ps => ("Энтропия s, кДж/(кг·К)", "Давление p, МПа"),
        DiagramKind::Th => ("Энтальпия h, кДж/кг", "Температура T, К"),
    };

    if swap_axes { (y, x) } else { (x, y) }
}

fn default_limits(kind: DiagramKind, swap_axes: bool) -> (f64, f64, f64, f64) {
    let (mut x_min, mut x_max, mut y_min, mut y_max) = match kind {
        DiagramKind::Pt => (273.15, 1000.0, 0.001, 100.0),
        DiagramKind::Pv => (0.001, 2.0, 0.001, 100.0),
        DiagramKind::Ps => (0.0, 10.0, 0.001, 100.0),
        DiagramKind::Ph => (0.0, 4000.0, 0.001, 100.0),
        DiagramKind::Tv => (0.001, 2.0, 273.15, 1000.0),
        DiagramKind::Ts => (0.0, 10.0, 273.15, 1000.0),
        DiagramKind::Th => (0.0, 4000.0, 273.15, 1000.0),
        DiagramKind::Hs => (0.0, 10.0, 0.0, 4000.0),
    };

    if swap_axes {
        std::mem::swap(&mut x_min, &mut y_min);
        std::mem::swap(&mut x_max, &mut y_max);
    }

    (x_min, x_max, y_min, y_max)
}

const SAT_P_MIN: f64 = 0.000611657_f64;
const SAT_P_MAX: f64 = 22.064_f64;
const SAT_MAX_DEPTH: usize = 12;
const SAT_REL_TOL_DEFAULT: f64 = 0.00035;
const SAT_REL_TOL_V: f64 = 0.00015;
const SAT_PRE_SAMPLES: usize = 32;

static DOME_CACHE: OnceLock<Mutex<HashMap<(DiagramKind, bool), Arc<Vec<(f64, f64)>>>>> =
    OnceLock::new();

fn saturation_pressure(t: f64) -> f64 {
    (SAT_P_MIN.ln() + t * (SAT_P_MAX.ln() - SAT_P_MIN.ln())).exp()
}

fn eval_saturation_point(
    kind: DiagramKind,
    swap_axes: bool,
    quality: f64,
    t: f64,
) -> Option<(f64, f64)> {
    let pressure = saturation_pressure(t);
    let state = if quality <= 0.0 {
        saturation::saturated_liquid(pressure.into()).ok()?
    } else {
        saturation::saturated_vapor(pressure.into()).ok()?
    };
    Some(project_state(kind, swap_axes, &state))
}

fn estimate_dome_spans(kind: DiagramKind, swap_axes: bool) -> (f64, f64, f64, f64) {
    let mut min_lin_x = f64::INFINITY;
    let mut max_lin_x = f64::NEG_INFINITY;
    let mut min_lin_y = f64::INFINITY;
    let mut max_lin_y = f64::NEG_INFINITY;

    let mut min_met_x = f64::INFINITY;
    let mut max_met_x = f64::NEG_INFINITY;
    let mut min_met_y = f64::INFINITY;
    let mut max_met_y = f64::NEG_INFINITY;

    let (x_var, y_var) = axis_vars(kind, swap_axes);
    let qualities = [0.0, 1.0];
    for &quality in &qualities {
        for i in 0..SAT_PRE_SAMPLES {
            let t = if SAT_PRE_SAMPLES <= 1 {
                0.0
            } else {
                i as f64 / (SAT_PRE_SAMPLES - 1) as f64
            };
            if let Some((x, y)) = eval_saturation_point(kind, swap_axes, quality, t) {
                if x.is_finite() && y.is_finite() {
                    min_lin_x = min_lin_x.min(x);
                    max_lin_x = max_lin_x.max(x);
                    min_lin_y = min_lin_y.min(y);
                    max_lin_y = max_lin_y.max(y);

                    let mx = metric_value(x_var, x);
                    let my = metric_value(y_var, y);
                    min_met_x = min_met_x.min(mx);
                    max_met_x = max_met_x.max(mx);
                    min_met_y = min_met_y.min(my);
                    max_met_y = max_met_y.max(my);
                }
            }
        }
    }

    if !min_lin_x.is_finite() || !min_lin_y.is_finite() {
        return (1.0, 1.0, 1.0, 1.0);
    }

    let lin_span_x = (max_lin_x - min_lin_x).abs().max(1e-9);
    let lin_span_y = (max_lin_y - min_lin_y).abs().max(1e-9);

    let met_span_x = if min_met_x.is_finite() {
        (max_met_x - min_met_x).abs().max(1e-9)
    } else {
        lin_span_x
    };
    let met_span_y = if min_met_y.is_finite() {
        (max_met_y - min_met_y).abs().max(1e-9)
    } else {
        lin_span_y
    };

    (lin_span_x, lin_span_y, met_span_x, met_span_y)
}

fn refine_saturation_segment(
    kind: DiagramKind,
    swap_axes: bool,
    quality: f64,
    x_var: AxisVar,
    y_var: AxisVar,
    lin_span_x: f64,
    lin_span_y: f64,
    met_span_x: f64,
    met_span_y: f64,
    t0: f64,
    p0: (f64, f64),
    t1: f64,
    p1: (f64, f64),
    depth: usize,
    out: &mut Vec<(f64, f64)>,
) {
    if depth >= SAT_MAX_DEPTH {
        out.push(p0);
        return;
    }

    let mid_t = 0.5 * (t0 + t1);
    let Some(pm) = eval_saturation_point(kind, swap_axes, quality, mid_t) else {
        out.push(p0);
        return;
    };

    let lin_dx = (p1.0 - p0.0) / lin_span_x;
    let lin_dy = (p1.1 - p0.1) / lin_span_y;
    let lin_seg_len = (lin_dx * lin_dx + lin_dy * lin_dy).sqrt();
    let lin_mx = (pm.0 - p0.0) / lin_span_x;
    let lin_my = (pm.1 - p0.1) / lin_span_y;
    let lin_dev = if lin_seg_len <= f64::EPSILON {
        0.0
    } else {
        (lin_dx * lin_my - lin_dy * lin_mx).abs() / lin_seg_len
    };

    let m0x = metric_value(x_var, p0.0);
    let m0y = metric_value(y_var, p0.1);
    let m1x = metric_value(x_var, p1.0);
    let m1y = metric_value(y_var, p1.1);
    let mmx = metric_value(x_var, pm.0);
    let mmy = metric_value(y_var, pm.1);

    let met_dx = (m1x - m0x) / met_span_x;
    let met_dy = (m1y - m0y) / met_span_y;
    let met_seg_len = (met_dx * met_dx + met_dy * met_dy).sqrt();
    let met_mx = (mmx - m0x) / met_span_x;
    let met_my = (mmy - m0y) / met_span_y;
    let met_dev = if met_seg_len <= f64::EPSILON {
        0.0
    } else {
        (met_dx * met_my - met_dy * met_mx).abs() / met_seg_len
    };

    let rel_tol = if matches!(x_var, AxisVar::V) || matches!(y_var, AxisVar::V) {
        SAT_REL_TOL_V
    } else {
        SAT_REL_TOL_DEFAULT
    };
    let need_refine = (lin_seg_len > 0.0 && lin_dev > lin_seg_len * rel_tol)
        || (met_seg_len > 0.0 && met_dev > met_seg_len * rel_tol);

    if need_refine {
        refine_saturation_segment(
            kind,
            swap_axes,
            quality,
            x_var,
            y_var,
            lin_span_x,
            lin_span_y,
            met_span_x,
            met_span_y,
            t0,
            p0,
            mid_t,
            pm,
            depth + 1,
            out,
        );
        refine_saturation_segment(
            kind,
            swap_axes,
            quality,
            x_var,
            y_var,
            lin_span_x,
            lin_span_y,
            met_span_x,
            met_span_y,
            mid_t,
            pm,
            t1,
            p1,
            depth + 1,
            out,
        );
    } else {
        out.push(p0);
    }
}

fn build_saturation_side(kind: DiagramKind, swap_axes: bool, quality: f64) -> Vec<(f64, f64)> {
    let Some(p0) = eval_saturation_point(kind, swap_axes, quality, 0.0) else {
        return Vec::new();
    };
    let Some(p1) = eval_saturation_point(kind, swap_axes, quality, 1.0) else {
        return Vec::new();
    };

    let (x_var, y_var) = axis_vars(kind, swap_axes);
    let (lin_span_x, lin_span_y, met_span_x, met_span_y) = estimate_dome_spans(kind, swap_axes);
    let mut points = Vec::new();
    refine_saturation_segment(
        kind,
        swap_axes,
        quality,
        x_var,
        y_var,
        lin_span_x,
        lin_span_y,
        met_span_x,
        met_span_y,
        0.0,
        p0,
        1.0,
        p1,
        0,
        &mut points,
    );
    points.push(p1);
    points
}

fn compute_dome_points(kind: DiagramKind, swap_axes: bool) -> Arc<Vec<(f64, f64)>> {
    let liquid = build_saturation_side(kind, swap_axes, 0.0);
    if liquid.is_empty() {
        return Arc::new(Vec::new());
    }

    if kind == DiagramKind::Pt {
        return Arc::new(liquid);
    }

    let mut vapor = build_saturation_side(kind, swap_axes, 1.0);
    vapor.reverse();

    let mut dome = liquid;
    dome.extend(vapor);
    Arc::new(dome)
}

fn dome_points_cached(kind: DiagramKind, swap_axes: bool) -> Arc<Vec<(f64, f64)>> {
    let cache = DOME_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(mut cache) = cache.lock() {
        if let Some(points) = cache.get(&(kind, swap_axes)) {
            return points.clone();
        }
        let points = compute_dome_points(kind, swap_axes);
        cache.insert((kind, swap_axes), points.clone());
        return points;
    }
    compute_dome_points(kind, swap_axes)
}

fn collect_limits(state: &AppState) -> (f64, f64, f64, f64) {
    if !state.autoscale {
        let (x_min, mut x_max, y_min, mut y_max) = state.custom_limits;
        if x_max <= x_min {
            x_max = x_min + 1.0;
        }
        if y_max <= y_min {
            y_max = y_min + 1.0;
        }
        return (x_min, x_max, y_min, y_max);
    }

    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for dataset in state.datasets.iter().filter(|dataset| dataset.visible) {
        for point in &dataset.points {
            let (x, y) = project_state(state.plot_type, state.swap_axes, point);
            if !x.is_finite() || !y.is_finite() {
                continue;
            }
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }
    }

    if !min_x.is_finite() || !min_y.is_finite() {
        return default_limits(state.plot_type, state.swap_axes);
    }

    let pad_x = ((max_x - min_x).abs() * 0.1).max(1e-6);
    let pad_y = ((max_y - min_y).abs() * 0.1).max(1e-6);
    (min_x - pad_x, max_x + pad_x, min_y - pad_y, max_y + pad_y)
}

/// Рендерит текущую диаграмму в RGB-буфер (24-bit).
pub fn render_plot_to_buffer(state: &AppState, width: u32, height: u32) -> Vec<u8> {
    let mut buffer = vec![0u8; (width * height * 3) as usize];
    {
        let root = BitMapBackend::with_buffer(&mut buffer, (width, height)).into_drawing_area();
        root.fill(&WHITE).ok();
        draw_core(state, &root);
    }
    buffer
}

/// Рендерит текущую диаграмму в файл.
pub fn render_plot_to_file(
    state: &AppState,
    filename: &str,
    width: u32,
    height: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(filename, (width, height)).into_drawing_area();
    root.fill(&WHITE)?;
    draw_core(state, &root);
    root.present()?;
    Ok(())
}

fn draw_core<DB: DrawingBackend>(state: &AppState, root: &DrawingArea<DB, plotters::coord::Shift>) {
    let (x_min, x_max, y_min, y_max) = collect_limits(state);
    let (x_desc, y_desc) = axis_labels(state.plot_type, state.swap_axes);

    let mut chart = match ChartBuilder::on(root)
        .margin(36)
        .x_label_area_size(56)
        .y_label_area_size(82)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)
    {
        Ok(chart) => chart,
        Err(_) => return,
    };

    chart
        .configure_mesh()
        .x_desc(x_desc)
        .y_desc(y_desc)
        .light_line_style(WHITE.mix(0.7))
        .draw()
        .ok();

    if state.show_dome {
        let dome_points = dome_points_cached(state.plot_type, state.swap_axes);
        if !dome_points.is_empty() {
            chart
                .draw_series(LineSeries::new(
                    dome_points.iter().copied(),
                    RGBColor(180, 0, 255).stroke_width(2),
                ))
                .ok();
        }
    }

    let mut color_idx = 0usize;
    for dataset in state.datasets.iter().filter(|dataset| dataset.visible) {
        if dataset.points.is_empty() {
            continue;
        }

        let color = get_palette_color(color_idx);
        color_idx += 1;
        let radius = if dataset.points.len() > 1000 { 2 } else { 4 };

        chart
            .draw_series(dataset.points.iter().filter_map(|point| {
                let (x, y) = project_state(state.plot_type, state.swap_axes, point);
                if !x.is_finite() || !y.is_finite() {
                    return None;
                }
                if x < x_min || x > x_max || y < y_min || y > y_max {
                    return None;
                }
                Some(Circle::new((x, y), radius, color.filled()))
            }))
            .ok()
            .map(|series| {
                series
                    .label(dataset.name.clone())
                    .legend(move |(x, y)| Circle::new((x, y), 4, color.filled()))
            });
    }

    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::UpperRight)
        .background_style(WHITE.mix(0.9))
        .border_style(BLACK)
        .draw()
        .ok();
}
