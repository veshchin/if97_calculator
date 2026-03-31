/* File: src/plot.rs */
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use crate::types::ChartType;
use if97_core::If97;
use std::cell::RefCell;

const INSIDE: u8 = 0; const LEFT: u8 = 1; const RIGHT: u8 = 2; const BOTTOM: u8 = 4; const TOP: u8 = 8;

fn compute_out_code(x: f64, y: f64, x_min: f64, x_max: f64, y_min: f64, y_max: f64) -> u8 {
    let mut code = INSIDE;
    if x < x_min { code |= LEFT; } else if x > x_max { code |= RIGHT; }
    if y < y_min { code |= BOTTOM; } else if y > y_max { code |= TOP; }
    code
}

pub fn cohen_sutherland(mut x0: f64, mut y0: f64, mut x1: f64, mut y1: f64, x_min: f64, x_max: f64, y_min: f64, y_max: f64) -> Option<((f64, f64), (f64, f64))> {
    let mut outcode0 = compute_out_code(x0, y0, x_min, x_max, y_min, y_max);
    let mut outcode1 = compute_out_code(x1, y1, x_min, x_max, y_min, y_max);
    let mut accept = false;

    loop {
        if outcode0 == 0 && outcode1 == 0 { accept = true; break; }
        else if (outcode0 & outcode1) != 0 { break; }
        else {
            let outcode_out = if outcode0 != 0 { outcode0 } else { outcode1 };
            let (mut x, mut y) = (0.0, 0.0);
            if outcode_out & TOP != 0 { x = x0 + (x1 - x0) * (y_max - y0) / (y1 - y0); y = y_max; }
            else if outcode_out & BOTTOM != 0 { x = x0 + (x1 - x0) * (y_min - y0) / (y1 - y0); y = y_min; }
            else if outcode_out & RIGHT != 0 { y = y0 + (y1 - y0) * (x_max - x0) / (x1 - x0); x = x_max; }
            else if outcode_out & LEFT != 0 { y = y0 + (y1 - y0) * (x_min - x0) / (x1 - x0); x = x_min; }

            if outcode_out == outcode0 { x0 = x; y0 = y; outcode0 = compute_out_code(x0, y0, x_min, x_max, y_min, y_max); }
            else { x1 = x; y1 = y; outcode1 = compute_out_code(x1, y1, x_min, x_max, y_min, y_max); }
        }
    }
    if accept { Some(((x0, y0), (x1, y1))) } else { None }
}

thread_local! {
    static DOME_CACHE: RefCell<Option<(ChartType, bool, Vec<(f64, f64)>)>> = RefCell::new(None);
}

pub struct ChartOptions {
    pub chart_type: ChartType,
    pub swap_axes: bool,
    pub draw_lines: bool,
    pub show_dome: bool,
    pub x_range: (f64, f64),
    pub y_range: (f64, f64),
}

#[derive(Clone, PartialEq)]
pub struct PlotSeries {
    pub name: String,
    pub points: Vec<(f64, f64)>,
}

pub fn draw_diagram(canvas_id: &str, opts: &ChartOptions, series_list: Vec<PlotSeries>) -> Result<(), Box<dyn std::error::Error>> {
    let backend = CanvasBackend::new(canvas_id).ok_or("Нет canvas")?;
    let root = backend.into_drawing_area();
    root.fill(&WHITE)?;

    let (x_min, x_max) = (opts.x_range.0.min(opts.x_range.1), opts.x_range.0.max(opts.x_range.1));
    let (y_min, y_max) = (opts.y_range.0.min(opts.y_range.1), opts.y_range.0.max(opts.y_range.1));

    let mut chart = ChartBuilder::on(&root)
        .margin(60)
        .x_label_area_size(100)
        .y_label_area_size(150)
        .build_cartesian_2d(opts.x_range.0..opts.x_range.1, opts.y_range.0..opts.y_range.1)?;

    chart.configure_mesh()
        .label_style(("sans-serif", 45).into_font())
        .light_line_style(&WHITE.mix(0.8))
        .draw()?;

    if opts.show_dome {
        let mut dome_full = Vec::new();

        DOME_CACHE.with(|cache| {
            let mut c = cache.borrow_mut();
            if let Some((ct, swap, ref data)) = *c {
                if ct == opts.chart_type && swap == opts.swap_axes {
                    dome_full = data.clone();
                }
            }

            if dome_full.is_empty() {
                let mut dome_liquid = Vec::new();
                let mut dome_vapor = Vec::new();
                let p_min = 0.000611657_f64; let p_max = 22.064_f64; let steps = 200;

                for i in 0..=steps {
                    let t = i as f64 / steps as f64;
                    let p_val = (p_min.ln() + t * (p_max.ln() - p_min.ln())).exp();
                    let get_coord = |state: if97_core::WaterState| -> (f64, f64) {
                        let (mut x, mut y) = match opts.chart_type {
                            ChartType::Ts => (state.s.inner(), state.t.inner()), ChartType::Hs => (state.s.inner(), state.h.inner()),
                            ChartType::Ph => (state.h.inner(), state.p.inner()), ChartType::Tv => (state.v.inner(), state.t.inner()),
                            ChartType::Pv => (state.v.inner(), state.p.inner()), ChartType::Pt => (state.t.inner(), state.p.inner()),
                        };
                        if opts.swap_axes { std::mem::swap(&mut x, &mut y); }
                        (x, y)
                    };
                    if let Ok(state_liq) = If97::px(p_val.into(), 0.0.into()) { dome_liquid.push(get_coord(state_liq)); }
                    if let Ok(state_vap) = If97::px(p_val.into(), 1.0.into()) { dome_vapor.push(get_coord(state_vap)); }
                }

                dome_vapor.reverse();
                dome_full = dome_liquid;
                dome_full.extend(dome_vapor);

                *c = Some((opts.chart_type, opts.swap_axes, dome_full.clone()));
            }
        });

        // Динамическая обрезка купола перед отрисовкой
        let mut clipped_dome = Vec::new();
        for i in 0..dome_full.len().saturating_sub(1) {
            let p0 = dome_full[i]; let p1 = dome_full[i + 1];
            if let Some((cp0, cp1)) = cohen_sutherland(p0.0, p0.1, p1.0, p1.1, x_min, x_max, y_min, y_max) {
                if clipped_dome.is_empty() { clipped_dome.push(cp0); }
                clipped_dome.push(cp1);
            }
        }
        if !clipped_dome.is_empty() {
            chart.draw_series(LineSeries::new(clipped_dome, RGBColor(128, 0, 128).stroke_width(3)))?;
        }
    }

    let colors = [BLUE, RED, GREEN, MAGENTA, CYAN, BLACK];

    for (i, series) in series_list.iter().enumerate() {
        let color = colors[i % colors.len()];

        if opts.draw_lines && series.points.len() > 1 {
            // Динамическая обрезка линий графиков
            let mut clipped_lines = Vec::new();
            for j in 0..series.points.len().saturating_sub(1) {
                let p0 = series.points[j]; let p1 = series.points[j + 1];
                if let Some((cp0, cp1)) = cohen_sutherland(p0.0, p0.1, p1.0, p1.1, x_min, x_max, y_min, y_max) {
                    if clipped_lines.is_empty() { clipped_lines.push(cp0); }
                    clipped_lines.push(cp1);
                }
            }
            if !clipped_lines.is_empty() {
                chart.draw_series(LineSeries::new(clipped_lines, color.stroke_width(3)))?;
            }
        }

        let pt_size = if series.points.len() > 1000 { 2 } else { 6 };

        chart.draw_series(
            series.points.iter().cloned()
                .filter(|&(x, y)| x >= x_min && x <= x_max && y >= y_min && y <= y_max)
                .map(|coord| Circle::new(coord, pt_size, color.filled()))
        )?;
    }

    root.present()?;
    Ok(())
}