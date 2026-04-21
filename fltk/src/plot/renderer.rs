use crate::state::AppState;
use if97_app_api::AxisVar;
use if97_core::{Region, WaterState, saturation};
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

fn default_range(var: AxisVar) -> (f64, f64) {
    match var {
        AxisVar::P => (0.001, 100.0),
        AxisVar::T => (273.15, 1000.0),
        AxisVar::V => (0.001, 2.0),
        AxisVar::Rho => (1.0, 1200.0),
        AxisVar::H => (0.0, 4000.0),
        AxisVar::S => (0.0, 10.0),
        AxisVar::U => (0.0, 3500.0),
        AxisVar::Cp => (0.0, 50.0),
        AxisVar::W => (0.0, 2000.0),
        AxisVar::X => (0.0, 1.0),
    }
}

fn axis_desc_ru(var: AxisVar) -> &'static str {
    match var {
        AxisVar::P => "Давление p, МПа",
        AxisVar::T => "Температура T, К",
        AxisVar::V => "Удельный объем v, м3/кг",
        AxisVar::Rho => "Плотность rho, кг/м3",
        AxisVar::H => "Энтальпия h, кДж/кг",
        AxisVar::S => "Энтропия s, кДж/(кг·К)",
        AxisVar::U => "Внутренняя энергия u, кДж/кг",
        AxisVar::Cp => "Изобарная теплоемкость cp, кДж/(кг·К)",
        AxisVar::W => "Скорость звука w, м/с",
        AxisVar::X => "Степень сухости x",
    }
}

fn sanitize_cp(value: f64) -> f64 {
    if value.is_finite() {
        return value;
    }

    // Для графика тоже не хотим NaN/inf: подменяем на большое конечное число,
    // если не удаётся посчитать разумное приближение.
    const CAP: f64 = 1e12;
    if value.is_sign_negative() { -CAP } else { CAP }
}

fn quality_x(state: &WaterState) -> f64 {
    if state.region != Region::Region4 {
        return f64::NAN;
    }

    let p = state.p;
    let v = state.v.inner();
    let Ok(liq) = saturation::saturated_liquid(p) else {
        return f64::NAN;
    };
    let Ok(vap) = saturation::saturated_vapor(p) else {
        return f64::NAN;
    };

    let v_liq = liq.v.inner();
    let v_vap = vap.v.inner();
    let denom = v_vap - v_liq;
    if !denom.is_finite() || denom.abs() < 1e-15 {
        return f64::NAN;
    }

    ((v - v_liq) / denom).clamp(0.0, 1.0)
}

fn axis_value(var: AxisVar, state: &WaterState) -> f64 {
    match var {
        AxisVar::P => state.p.inner(),
        AxisVar::T => state.t.inner(),
        AxisVar::V => state.v.inner(),
        AxisVar::Rho => state.rho.inner(),
        AxisVar::H => state.h.inner(),
        AxisVar::S => state.s.inner(),
        AxisVar::U => state.u.inner(),
        AxisVar::Cp => {
            let raw = state.cp.inner();
            if raw.is_finite() {
                raw
            } else if state.region == Region::Region4 {
                let x = quality_x(state);
                if !x.is_finite() {
                    sanitize_cp(raw)
                } else {
                    let p = state.p;
                    let Ok(liq) = saturation::saturated_liquid(p) else {
                        return sanitize_cp(raw);
                    };
                    let Ok(vap) = saturation::saturated_vapor(p) else {
                        return sanitize_cp(raw);
                    };
                    let cp = liq.cp.inner() + x * (vap.cp.inner() - liq.cp.inner());
                    if cp.is_finite() { cp } else { sanitize_cp(raw) }
                }
            } else {
                sanitize_cp(raw)
            }
        }
        AxisVar::W => state.w.inner(),
        AxisVar::X => quality_x(state),
    }
}

fn axis_value_dome(var: AxisVar, state: &WaterState, quality: f64) -> f64 {
    if matches!(var, AxisVar::X) {
        return quality;
    }
    axis_value(var, state)
}

fn metric_value(var: AxisVar, value: f64) -> f64 {
    // Для v-диаграмм размах по оси v большой, из-за чего адаптивное уточнение
    // недодаёт точек в области малого v. В метрике используем ln(v).
    match var {
        AxisVar::V => value.max(1e-12).ln(),
        _ => value,
    }
}

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

fn plot_range_from_raw(min_raw: f64, max_raw: f64, log: bool) -> (f64, f64) {
    let (mut min, mut max) = if log {
        let (min, max) = clamp_positive_range(min_raw, max_raw);
        (min.log10(), max.log10())
    } else {
        (min_raw.min(max_raw), min_raw.max(max_raw))
    };

    if (max - min).abs() < 1e-12 {
        min -= 1.0;
        max += 1.0;
    }

    (min, max)
}

fn transform(value: f64, log: bool) -> Option<f64> {
    if !value.is_finite() {
        return None;
    }
    if log {
        if value <= 0.0 {
            return None;
        }
        Some(value.log10())
    } else {
        Some(value)
    }
}

const SAT_P_MIN: f64 = 0.000611657_f64;
const SAT_P_MAX: f64 = 22.064_f64;
const SAT_MAX_DEPTH: usize = 12;
const SAT_REL_TOL_DEFAULT: f64 = 0.00035;
const SAT_REL_TOL_V: f64 = 0.00015;
const SAT_PRE_SAMPLES: usize = 32;

static DOME_CACHE: OnceLock<Mutex<HashMap<(AxisVar, AxisVar), Arc<Vec<(f64, f64)>>>>> =
    OnceLock::new();

fn saturation_pressure(t: f64) -> f64 {
    (SAT_P_MIN.ln() + t * (SAT_P_MAX.ln() - SAT_P_MIN.ln())).exp()
}

fn eval_saturation_point(
    x_var: AxisVar,
    y_var: AxisVar,
    quality: f64,
    t: f64,
) -> Option<(f64, f64)> {
    let pressure = saturation_pressure(t);
    let state = if quality <= 0.0 {
        saturation::saturated_liquid(pressure.into()).ok()?
    } else {
        saturation::saturated_vapor(pressure.into()).ok()?
    };
    Some((
        axis_value_dome(x_var, &state, quality),
        axis_value_dome(y_var, &state, quality),
    ))
}

fn estimate_dome_spans(x_var: AxisVar, y_var: AxisVar) -> (f64, f64, f64, f64) {
    let mut min_lin_x = f64::INFINITY;
    let mut max_lin_x = f64::NEG_INFINITY;
    let mut min_lin_y = f64::INFINITY;
    let mut max_lin_y = f64::NEG_INFINITY;

    let mut min_met_x = f64::INFINITY;
    let mut max_met_x = f64::NEG_INFINITY;
    let mut min_met_y = f64::INFINITY;
    let mut max_met_y = f64::NEG_INFINITY;

    let qualities = [0.0, 1.0];
    for &quality in &qualities {
        for i in 0..SAT_PRE_SAMPLES {
            let t = if SAT_PRE_SAMPLES <= 1 {
                0.0
            } else {
                i as f64 / (SAT_PRE_SAMPLES - 1) as f64
            };
            if let Some((x, y)) = eval_saturation_point(x_var, y_var, quality, t) {
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
    x_var: AxisVar,
    y_var: AxisVar,
    quality: f64,
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
    let Some(pm) = eval_saturation_point(x_var, y_var, quality, mid_t) else {
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
            x_var,
            y_var,
            quality,
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
            x_var,
            y_var,
            quality,
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

fn build_saturation_side(x_var: AxisVar, y_var: AxisVar, quality: f64) -> Vec<(f64, f64)> {
    let Some(p0) = eval_saturation_point(x_var, y_var, quality, 0.0) else {
        return Vec::new();
    };
    let Some(p1) = eval_saturation_point(x_var, y_var, quality, 1.0) else {
        return Vec::new();
    };

    let (lin_span_x, lin_span_y, met_span_x, met_span_y) = estimate_dome_spans(x_var, y_var);
    let mut points = Vec::new();
    refine_saturation_segment(
        x_var,
        y_var,
        quality,
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

fn compute_dome_points(x_var: AxisVar, y_var: AxisVar) -> Arc<Vec<(f64, f64)>> {
    let liquid = build_saturation_side(x_var, y_var, 0.0);
    if liquid.is_empty() {
        return Arc::new(Vec::new());
    }

    // В p-T (и T-p) диаграммах линия насыщения совпадает для x=0 и x=1, поэтому не дублируем путь.
    if (x_var == AxisVar::T && y_var == AxisVar::P) || (x_var == AxisVar::P && y_var == AxisVar::T)
    {
        return Arc::new(liquid);
    }

    let mut vapor = build_saturation_side(x_var, y_var, 1.0);
    vapor.reverse();

    let mut dome = liquid;
    dome.extend(vapor);
    Arc::new(dome)
}

fn dome_points_cached(x_var: AxisVar, y_var: AxisVar) -> Arc<Vec<(f64, f64)>> {
    let cache = DOME_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(mut cache) = cache.lock() {
        if let Some(points) = cache.get(&(x_var, y_var)) {
            return points.clone();
        }
        let points = compute_dome_points(x_var, y_var);
        cache.insert((x_var, y_var), points.clone());
        return points;
    }
    compute_dome_points(x_var, y_var)
}

fn collect_raw_limits(state: &AppState) -> (f64, f64, f64, f64) {
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
            let x = axis_value(state.plot_x, point);
            let y = axis_value(state.plot_y, point);
            if !x.is_finite() || !y.is_finite() {
                continue;
            }
            if state.plot_x_log && x <= 0.0 {
                continue;
            }
            if state.plot_y_log && y <= 0.0 {
                continue;
            }

            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }
    }

    if !min_x.is_finite() || !min_y.is_finite() {
        let (x_min, x_max) = default_range(state.plot_x);
        let (y_min, y_max) = default_range(state.plot_y);
        return (x_min, x_max, y_min, y_max);
    }

    let pad_x = ((max_x - min_x).abs() * 0.1).max(1e-6);
    let pad_y = ((max_y - min_y).abs() * 0.1).max(1e-6);
    (min_x - pad_x, max_x + pad_x, min_y - pad_y, max_y + pad_y)
}

fn dome_segments(dome: &[(f64, f64)], x_log: bool, y_log: bool) -> Vec<Vec<(f64, f64)>> {
    let mut segments: Vec<Vec<(f64, f64)>> = Vec::new();
    let mut current: Vec<(f64, f64)> = Vec::new();

    for &(x, y) in dome {
        let Some(tx) = transform(x, x_log) else {
            if current.len() >= 2 {
                segments.push(std::mem::take(&mut current));
            } else {
                current.clear();
            }
            continue;
        };
        let Some(ty) = transform(y, y_log) else {
            if current.len() >= 2 {
                segments.push(std::mem::take(&mut current));
            } else {
                current.clear();
            }
            continue;
        };
        current.push((tx, ty));
    }

    if current.len() >= 2 {
        segments.push(current);
    }

    segments
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

/// Рендерит текущую диаграмму в SVG-файл.
pub fn render_plot_to_svg_file(
    state: &AppState,
    filename: &str,
    width: u32,
    height: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = SVGBackend::new(filename, (width, height)).into_drawing_area();
    root.fill(&WHITE)?;
    draw_core(state, &root);
    root.present()?;
    Ok(())
}

fn draw_core<DB: DrawingBackend>(state: &AppState, root: &DrawingArea<DB, plotters::coord::Shift>) {
    let (x_min_raw, x_max_raw, y_min_raw, y_max_raw) = collect_raw_limits(state);

    let (x_min, x_max) = plot_range_from_raw(x_min_raw, x_max_raw, state.plot_x_log);
    let (y_min, y_max) = plot_range_from_raw(y_min_raw, y_max_raw, state.plot_y_log);

    let mut chart = match ChartBuilder::on(root)
        .margin(36)
        .x_label_area_size(56)
        .y_label_area_size(82)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)
    {
        Ok(chart) => chart,
        Err(_) => return,
    };

    {
        let mut mesh = chart.configure_mesh();
        mesh.x_desc(axis_desc_ru(state.plot_x))
            .y_desc(axis_desc_ru(state.plot_y))
            .light_line_style(WHITE.mix(0.7));
        if state.plot_x_log {
            mesh.x_label_formatter(&|v| format!("{:.3e}", 10_f64.powf(*v)));
        }
        if state.plot_y_log {
            mesh.y_label_formatter(&|v| format!("{:.3e}", 10_f64.powf(*v)));
        }
        mesh.draw().ok();
    }

    if state.show_dome {
        let dome_points = dome_points_cached(state.plot_x, state.plot_y);
        let segments = dome_segments(dome_points.as_slice(), state.plot_x_log, state.plot_y_log);
        for seg in segments.into_iter() {
            chart
                .draw_series(LineSeries::new(
                    seg.into_iter(),
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

        let x_log = state.plot_x_log;
        let y_log = state.plot_y_log;

        chart
            .draw_series(dataset.points.iter().filter_map(|point| {
                let x = axis_value(state.plot_x, point);
                let y = axis_value(state.plot_y, point);
                let tx = transform(x, x_log)?;
                let ty = transform(y, y_log)?;
                if tx < x_min || tx > x_max || ty < y_min || ty > y_max {
                    return None;
                }
                Some(Circle::new((tx, ty), radius, color.filled()))
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
