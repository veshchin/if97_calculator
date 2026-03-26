// web_wasm/src/plot.rs

use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use crate::app::App;
use crate::types::PlotType;
use if97_core::domain::state::WaterState;
use if97_core::domain::calculator::Calculator;

fn get_palette_color(idx: usize) -> RGBColor {
    let palette = [RED, BLUE, GREEN, MAGENTA, CYAN, BLACK];
    palette[idx % palette.len()]
}

fn clip_line(
    mut x0: f64, mut y0: f64,
    mut x1: f64, mut y1: f64,
    min_x: f64, max_x: f64,
    min_y: f64, max_y: f64
) -> Option<((f64, f64), (f64, f64))> {
    let compute_outcode = |x: f64, y: f64| -> i32 {
        let mut code = 0;
        if x < min_x { code |= 1; } else if x > max_x { code |= 2; }
        if y < min_y { code |= 4; } else if y > max_y { code |= 8; }
        code
    };

    let mut outcode0 = compute_outcode(x0, y0);
    let mut outcode1 = compute_outcode(x1, y1);
    let mut accept = false;

    loop {
        if outcode0 == 0 && outcode1 == 0 {
            accept = true;
            break;
        } else if (outcode0 & outcode1) != 0 {
            break;
        } else {
            let outcode_out = if outcode0 != 0 { outcode0 } else { outcode1 };
            let mut x = 0.0;
            let mut y = 0.0;

            if (outcode_out & 8) != 0 {
                x = x0 + (x1 - x0) * (max_y - y0) / (y1 - y0);
                y = max_y;
            } else if (outcode_out & 4) != 0 {
                x = x0 + (x1 - x0) * (min_y - y0) / (y1 - y0);
                y = min_y;
            } else if (outcode_out & 2) != 0 {
                y = y0 + (y1 - y0) * (max_x - x0) / (x1 - x0);
                x = max_x;
            } else if (outcode_out & 1) != 0 {
                y = y0 + (y1 - y0) * (min_x - x0) / (x1 - x0);
                x = min_x;
            }

            if outcode_out == outcode0 {
                x0 = x; y0 = y; outcode0 = compute_outcode(x0, y0);
            } else {
                x1 = x; y1 = y; outcode1 = compute_outcode(x1, y1);
            }
        }
    }

    if accept {
        Some(((x0, y0), (x1, y1)))
    } else {
        None
    }
}

pub fn draw_chart(app: &App, canvas_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let backend = CanvasBackend::new(canvas_id).ok_or("Canvas не найден")?;
    let root = backend.into_drawing_area();
    root.fill(&WHITE)?;

    let get_coords = |s: &WaterState| -> (f64, f64) {
        let (x, y) = match app.plot_type {
            PlotType::PT => (s.p, s.t),
            PlotType::RhoT => (s.rho, s.t),
            PlotType::VT => (s.v, s.t),
            PlotType::PV => (s.v, s.p),
            PlotType::TS => (s.s, s.t),
            PlotType::HS => (s.s, s.h),
            PlotType::PH => (s.h, s.p),
        };

        if app.swap_axes {
            (y, x)
        } else {
            (x, y)
        }
    };

    let mut sat_liq = Vec::new();
    let mut sat_vap = Vec::new();

    if app.show_dome {
        let mut p = 0.000611;
        let p_crit = 22.064;
        while p <= p_crit {
            if let Ok(st) = Calculator::calculate_px(p, 0.0) { sat_liq.push(st); }
            if let Ok(st) = Calculator::calculate_px(p, 1.0) { sat_vap.push(st); }
            if p < 0.01 { p += 0.002; } else if p < 0.1 { p += 0.02; } else if p < 1.0 { p += 0.2; } else if p < 10.0 { p += 1.0; } else { p += 2.0; }
        }
        if let Ok(st) = Calculator::calculate_px(p_crit, 0.5) { sat_liq.push(st.clone()); sat_vap.push(st); }
    }

    let mut min_x = f64::MAX; let mut max_x = f64::MIN;
    let mut min_y = f64::MAX; let mut max_y = f64::MIN;

    if app.autoscale {
        let mut valid_points = 0;
        for ds in &app.datasets {
            if !ds.visible { continue; }
            for s in &ds.points {
                let (cx, cy) = get_coords(s);
                if cx.is_nan() || cy.is_nan() { continue; }
                if cx < min_x { min_x = cx; } if cx > max_x { max_x = cx; }
                if cy < min_y { min_y = cy; } if cy > max_y { max_y = cy; }
                valid_points += 1;
            }
        }
        if valid_points == 0 {
            if app.show_dome {
                let (mut dx_min, mut dx_max) = match app.plot_type {
                    PlotType::PT   => (0.0, 25.0),     // Давление (МПа)
                    PlotType::RhoT => (0.0, 1100.0),   // Плотность (кг/м3)
                    PlotType::VT   => (0.0005, 0.2),   // Уд. объем (м3/кг)
                    PlotType::PV   => (0.0005, 0.2),   // Уд. объем (м3/кг) для P-v
                    PlotType::TS   => (0.0, 10.0),     // Энтропия (кДж/кгК)
                    PlotType::HS   => (0.0, 10.0),     // Энтропия (кДж/кгК) для h-s
                    PlotType::PH   => (0.0, 4000.0),   // Энтальпия (кДж/кг)
                };                let (mut dy_min, mut dy_max) = (270.0, 660.0);
                if app.swap_axes { std::mem::swap(&mut dx_min, &mut dy_min); std::mem::swap(&mut dx_max, &mut dy_max); }
                min_x = dx_min; max_x = dx_max; min_y = dy_min; max_y = dy_max;
            } else {
                min_x = 0.0; max_x = 100.0; min_y = 273.15; max_y = 2273.15;
                if app.swap_axes { std::mem::swap(&mut min_x, &mut min_y); std::mem::swap(&mut max_x, &mut max_y); }
            }
        } else {
            if max_x <= min_x { min_x -= 1.0; max_x += 1.0; } else { let px = (max_x - min_x) * 0.1; min_x -= px; max_x += px; }
            if max_y <= min_y { min_y -= 10.0; max_y += 10.0; } else { let py = (max_y - min_y) * 0.1; min_y -= py; max_y += py; }
        }
    } else {
        if app.swap_axes { min_x = app.t_min; max_x = app.t_max; min_y = app.val_min; max_y = app.val_max; }
        else { min_x = app.val_min; max_x = app.val_max; min_y = app.t_min; max_y = app.t_max; }
    }

    let mut chart = ChartBuilder::on(&root).margin(40).x_label_area_size(40).y_label_area_size(60).build_cartesian_2d(min_x..max_x, min_y..max_y)?;

    let desc_val = match app.plot_type {
        PlotType::PT   => "Давление, МПа",
        PlotType::RhoT => "Плотность, кг/м3",
        PlotType::VT   => "Уд. объем, м3/кг",
        PlotType::PV   => "Уд. объем, м3/кг",
        PlotType::TS   => "Энтропия, кДж/(кг·К)",
        PlotType::HS   => "Энтропия, кДж/(кг·К)",
        PlotType::PH   => "Энтальпия, кДж/кг",
    };    let desc_t = "Температура, К";
    let (x_desc, y_desc) = if app.swap_axes { (desc_t, desc_val) } else { (desc_val, desc_t) };

    chart.configure_mesh().x_desc(x_desc).y_desc(y_desc).draw()?;

    if app.show_dome {
        let bright_purple = RGBColor(180, 0, 255);
        let sat_style = ShapeStyle::from(&bright_purple).stroke_width(2);

        // Собираем все точки купола в один массив
        let dome_points = if app.plot_type == PlotType::PT {
            sat_liq.iter().map(|s| get_coords(s)).collect::<Vec<_>>()
        } else {
            let mut d: Vec<(f64, f64)> = sat_liq.iter().map(|s| get_coords(s)).collect();
            let mut v: Vec<(f64, f64)> = sat_vap.iter().map(|s| get_coords(s)).collect();
            v.reverse();
            d.extend(v);
            d
        };

        // Разбиваем купол на отрезки и математически обрезаем те, что выходят за бокс
        let segments = dome_points.windows(2).filter_map(|w| {
            let (x1, y1) = w[0];
            let (x2, y2) = w[1];

            // Обрезаем линию по границам графика
            if let Some((p1, p2)) = clip_line(x1, y1, x2, y2, min_x, max_x, min_y, max_y) {
                Some(PathElement::new(vec![p1, p2], sat_style.clone()))
            } else {
                None
            }
        });

        chart.draw_series(segments)?;
    }

    let mut color_idx = 0;
    for ds in &app.datasets {
        if !ds.visible || ds.points.is_empty() { continue; }
        let color = get_palette_color(color_idx);
        color_idx += 1;

        chart.draw_series(ds.points.iter().filter_map(|s| {
            let (cx, cy) = get_coords(s);
            if cx.is_nan() || cy.is_nan() || cx < min_x || cx > max_x || cy < min_y || cy > max_y { return None; }
            Some(Circle::new((cx, cy), 4, color.filled()))
        }))?
            .label(&ds.name)
            .legend(move |(x, y)| Circle::new((x, y), 4, color.filled()));
    }

    chart.configure_series_labels().position(SeriesLabelPosition::UpperRight).background_style(&WHITE.mix(0.8)).border_style(&BLACK).draw()?;
    root.present()?;
    Ok(())
}