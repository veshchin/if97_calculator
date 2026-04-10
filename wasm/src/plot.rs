//! Рендеринг диаграмм в `HtmlCanvasElement` через `plotters-canvas`.
use if97_app_api::{AxisVar, PlotPoint, StateDto};
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use web_sys::HtmlCanvasElement;

const INSIDE: u8 = 0;
const LEFT: u8 = 1;
const RIGHT: u8 = 2;
const BOTTOM: u8 = 4;
const TOP: u8 = 8;

fn compute_out_code(x: f64, y: f64, x_min: f64, x_max: f64, y_min: f64, y_max: f64) -> u8 {
    let mut code = INSIDE;
    if x < x_min {
        code |= LEFT;
    } else if x > x_max {
        code |= RIGHT;
    }
    if y < y_min {
        code |= BOTTOM;
    } else if y > y_max {
        code |= TOP;
    }
    code
}

/// Клиппинг отрезка по прямоугольнику (алгоритм Cohen-Sutherland).
pub fn cohen_sutherland(
    mut x0: f64,
    mut y0: f64,
    mut x1: f64,
    mut y1: f64,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
) -> Option<((f64, f64), (f64, f64))> {
    let mut outcode0 = compute_out_code(x0, y0, x_min, x_max, y_min, y_max);
    let mut outcode1 = compute_out_code(x1, y1, x_min, x_max, y_min, y_max);
    let mut accept = false;

    loop {
        if outcode0 == 0 && outcode1 == 0 {
            accept = true;
            break;
        } else if (outcode0 & outcode1) != 0 {
            break;
        } else {
            let outcode_out = if outcode0 != 0 { outcode0 } else { outcode1 };
            let (mut x, mut y) = (0.0, 0.0);
            if outcode_out & TOP != 0 {
                if (y1 - y0).abs() < f64::EPSILON {
                    return None;
                }
                x = x0 + (x1 - x0) * (y_max - y0) / (y1 - y0);
                y = y_max;
            } else if outcode_out & BOTTOM != 0 {
                if (y1 - y0).abs() < f64::EPSILON {
                    return None;
                }
                x = x0 + (x1 - x0) * (y_min - y0) / (y1 - y0);
                y = y_min;
            } else if outcode_out & RIGHT != 0 {
                if (x1 - x0).abs() < f64::EPSILON {
                    return None;
                }
                y = y0 + (y1 - y0) * (x_max - x0) / (x1 - x0);
                x = x_max;
            } else if outcode_out & LEFT != 0 {
                if (x1 - x0).abs() < f64::EPSILON {
                    return None;
                }
                y = y0 + (y1 - y0) * (x_min - x0) / (x1 - x0);
                x = x_min;
            }

            if outcode_out == outcode0 {
                x0 = x;
                y0 = y;
                outcode0 = compute_out_code(x0, y0, x_min, x_max, y_min, y_max);
            } else {
                x1 = x;
                y1 = y;
                outcode1 = compute_out_code(x1, y1, x_min, x_max, y_min, y_max);
            }
        }
    }
    if accept {
        Some(((x0, y0), (x1, y1)))
    } else {
        None
    }
}

/// Параметры построения диаграммы.
pub struct ChartOptions {
    /// Показывать купол насыщения.
    pub show_dome: bool,
    /// Диапазон оси X.
    pub x_range: (f64, f64),
    /// Диапазон оси Y.
    pub y_range: (f64, f64),
    /// Логарифмическая шкала по X.
    pub x_log: bool,
    /// Логарифмическая шкала по Y.
    pub y_log: bool,
}

#[derive(Clone, PartialEq)]
/// Серия точек для отрисовки на диаграмме.
pub struct PlotSeries {
    /// Имя серии (для UI).
    pub name: String,
    /// Набор точек (x, y) в координатах диаграммы.
    pub points: Vec<(f64, f64)>,
}

/// Отрисовывает диаграмму на canvas.
pub fn draw_diagram(
    canvas: &HtmlCanvasElement,
    opts: &ChartOptions,
    dome_points: &[PlotPoint],
    series_list: &[PlotSeries],
) -> Result<(), Box<dyn std::error::Error>> {
    let backend = CanvasBackend::with_canvas_object(canvas.clone()).ok_or("Холст не найден")?;
    let root = backend.into_drawing_area();
    root.fill(&WHITE)?;

    let (x_min_raw, x_max_raw) = (
        opts.x_range.0.min(opts.x_range.1),
        opts.x_range.0.max(opts.x_range.1),
    );
    let (y_min_raw, y_max_raw) = (
        opts.y_range.0.min(opts.y_range.1),
        opts.y_range.0.max(opts.y_range.1),
    );

    let clamp_positive_range = |mut min: f64, mut max: f64| -> (f64, f64) {
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
    };

    let (x_min_plot, x_max_plot) = if opts.x_log {
        let (min, max) = clamp_positive_range(x_min_raw, x_max_raw);
        (min.log10(), max.log10())
    } else {
        (x_min_raw, x_max_raw)
    };

    let (y_min_plot, y_max_plot) = if opts.y_log {
        let (min, max) = clamp_positive_range(y_min_raw, y_max_raw);
        (min.log10(), max.log10())
    } else {
        (y_min_raw, y_max_raw)
    };

    let (x_min, x_max) = if (x_max_plot - x_min_plot).abs() < 1e-12 {
        (x_min_plot - 1.0, x_max_plot + 1.0)
    } else {
        (x_min_plot, x_max_plot)
    };
    let (y_min, y_max) = if (y_max_plot - y_min_plot).abs() < 1e-12 {
        (y_min_plot - 1.0, y_max_plot + 1.0)
    } else {
        (y_min_plot, y_max_plot)
    };

    let transform = |value: f64, log: bool| -> Option<f64> {
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
    };

    let mut chart = ChartBuilder::on(&root)
        .margin(60)
        .x_label_area_size(100)
        .y_label_area_size(150)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;

    {
        let mut mesh = chart.configure_mesh();
        mesh.label_style(("sans-serif", 24).into_font())
            .light_line_style(&WHITE.mix(0.8));
        if opts.x_log {
            mesh.x_label_formatter(&|v| format!("{:.3e}", 10_f64.powf(*v)));
        }
        if opts.y_log {
            mesh.y_label_formatter(&|v| format!("{:.3e}", 10_f64.powf(*v)));
        }
        mesh.draw()?;
    }

    if opts.show_dome {
        // Быстрый путь: если все точки купола уже находятся в пределах текущих осей,
        // клиппинг не нужен и можно рисовать одну полилинию.
        let all_inside = dome_points.iter().all(|p| {
            let Some(tx) = transform(p.x, opts.x_log) else {
                return false;
            };
            let Some(ty) = transform(p.y, opts.y_log) else {
                return false;
            };
            tx >= x_min && tx <= x_max && ty >= y_min && ty <= y_max
        });

        if all_inside {
            chart.draw_series(LineSeries::new(
                dome_points
                    .iter()
                    .filter_map(|p| Some((transform(p.x, opts.x_log)?, transform(p.y, opts.y_log)?))),
                RGBColor(128, 0, 128).stroke_width(3),
            ))?;
        } else {
            // Рисуем купол несколькими сегментами. Если просто собрать все отрезки в один LineSeries,
            // то при клиппинге появятся "диагонали" между разорванными частями кривой.
            let mut segments: Vec<Vec<(f64, f64)>> = Vec::new();
            let mut current: Vec<(f64, f64)> = Vec::new();
            for i in 0..dome_points.len().saturating_sub(1) {
                let Some(p0) = transform(dome_points[i].x, opts.x_log)
                    .zip(transform(dome_points[i].y, opts.y_log))
                else {
                    if current.len() >= 2 {
                        segments.push(std::mem::take(&mut current));
                    } else {
                        current.clear();
                    }
                    continue;
                };
                let Some(p1) = transform(dome_points[i + 1].x, opts.x_log)
                    .zip(transform(dome_points[i + 1].y, opts.y_log))
                else {
                    if current.len() >= 2 {
                        segments.push(std::mem::take(&mut current));
                    } else {
                        current.clear();
                    }
                    continue;
                };
                if let Some((cp0, cp1)) =
                    cohen_sutherland(p0.0, p0.1, p1.0, p1.1, x_min, x_max, y_min, y_max)
                {
                    let need_start = current
                        .last()
                        .map(|last| {
                            (last.0 - cp0.0).abs() > 1e-12 || (last.1 - cp0.1).abs() > 1e-12
                        })
                        .unwrap_or(true);
                    if need_start {
                        current.push(cp0);
                    }
                    current.push(cp1);
                } else if current.len() >= 2 {
                    segments.push(std::mem::take(&mut current));
                } else {
                    current.clear();
                }
            }
            if current.len() >= 2 {
                segments.push(current);
            }

            for seg in segments.into_iter().filter(|seg| seg.len() >= 2) {
                chart.draw_series(LineSeries::new(
                    seg,
                    RGBColor(128, 0, 128).stroke_width(3),
                ))?;
            }
        }
    }

    let colors = [BLUE, RED, GREEN, MAGENTA, CYAN, BLACK];

    for (i, series) in series_list.iter().enumerate() {
        let color = colors[i % colors.len()];

        let pt_size = if series.points.len() > 1000 { 2 } else { 6 };

        chart.draw_series(
            series
                .points
                .iter()
                .cloned()
                .filter_map(|(x, y)| {
                    let tx = transform(x, opts.x_log)?;
                    let ty = transform(y, opts.y_log)?;
                    if tx < x_min || tx > x_max || ty < y_min || ty > y_max {
                        return None;
                    }
                    Some(Circle::new((tx, ty), pt_size, color.filled()))
                }),
        )?;
    }

    root.present()?;
    Ok(())
}

/// Проецирует `StateDto` в координаты диаграммы (x, y).
pub fn project_state(x_var: AxisVar, y_var: AxisVar, state: &StateDto) -> (f64, f64) {
    fn axis_value(var: AxisVar, state: &StateDto) -> f64 {
        match var {
            AxisVar::P => state.p,
            AxisVar::T => state.t,
            AxisVar::V => state.v,
            AxisVar::Rho => state.rho,
            AxisVar::H => state.h,
            AxisVar::S => state.s,
            AxisVar::U => state.u,
            AxisVar::Cp => state.cp,
            AxisVar::W => state.w,
            AxisVar::X => state.x,
        }
    }

    (axis_value(x_var, state), axis_value(y_var, state))
}
