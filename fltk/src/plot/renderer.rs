use crate::state::AppState;
use if97_app_api::DiagramKind;
use if97_core::{If97, WaterState};
use plotters::prelude::*;

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

fn build_dome_points(kind: DiagramKind, swap_axes: bool) -> Vec<(f64, f64)> {
    let mut liquid = Vec::new();
    let mut vapor = Vec::new();
    let p_min = 0.000611657_f64;
    let p_max = 22.064_f64;
    let steps = 220;

    for index in 0..=steps {
        let t = index as f64 / steps as f64;
        let pressure = (p_min.ln() + t * (p_max.ln() - p_min.ln())).exp();
        if let Ok(state) = If97::px(pressure.into(), 0.0.into()) {
            liquid.push(project_state(kind, swap_axes, &state));
        }
        if let Ok(state) = If97::px(pressure.into(), 1.0.into()) {
            vapor.push(project_state(kind, swap_axes, &state));
        }
    }

    vapor.reverse();
    liquid.extend(vapor);
    liquid
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

pub fn render_plot_to_buffer(state: &AppState, width: u32, height: u32) -> Vec<u8> {
    let mut buffer = vec![0u8; (width * height * 3) as usize];
    {
        let root = BitMapBackend::with_buffer(&mut buffer, (width, height)).into_drawing_area();
        root.fill(&WHITE).ok();
        draw_core(state, &root);
    }
    buffer
}

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
        let dome_points = build_dome_points(state.plot_type, state.swap_axes);
        if !dome_points.is_empty() {
            chart
                .draw_series(LineSeries::new(
                    dome_points,
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
