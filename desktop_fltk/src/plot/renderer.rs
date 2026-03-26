// desktop_fltk/src/plot/renderer.rs

use plotters::prelude::*;
use crate::state::{AppState, PlotType};
use if97_core::domain::calculator::Calculator;
use if97_core::domain::state::WaterState;

fn get_palette_color(idx: usize) -> RGBColor {
    let palette = [RED, BLUE, GREEN, MAGENTA, CYAN, BLACK];
    palette[idx % palette.len()]
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

pub fn render_plot_to_file(state: &AppState, filename: &str, width: u32, height: u32) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(filename, (width, height)).into_drawing_area();
    root.fill(&WHITE)?;
    draw_core(state, &root);
    root.present()?;
    Ok(())
}

fn draw_core<DB: DrawingBackend>(state: &AppState, root: &DrawingArea<DB, plotters::coord::Shift>) {
    let current_plot = state.plot_type;
    let show_dome = state.show_dome;
    let swap_axes = state.swap_axes;
    let autoscale = state.autoscale;
    let custom_limits = state.custom_limits;

    let bright_purple = RGBColor(180, 0, 255);

    let get_coords = |s: &WaterState| -> (f64, f64) {
        let val = match current_plot {
            PlotType::PT => s.p,
            PlotType::RhoT => s.rho,
            PlotType::VT => s.v,
        };
        if swap_axes { (s.t, val) } else { (val, s.t) }
    };

    let mut sat_liq = Vec::new();
    let mut sat_vap = Vec::new();

    if show_dome {
        let mut p = 0.000611;
        let p_crit = 22.064;
        while p <= p_crit {
            if let Ok(st) = Calculator::calculate_px(p, 0.0) { sat_liq.push(st); }
            if let Ok(st) = Calculator::calculate_px(p, 1.0) { sat_vap.push(st); }

            if p < 0.01 { p += 0.002; }
            else if p < 0.1 { p += 0.02; }
            else if p < 1.0 { p += 0.2; }
            else if p < 10.0 { p += 1.0; }
            else { p += 2.0; }
        }
        if let Ok(st) = Calculator::calculate_px(p_crit, 0.5) {
            sat_liq.push(st.clone());
            sat_vap.push(st);
        }
    }

    let mut min_x = f64::MAX;
    let mut max_x = f64::MIN;
    let mut min_y = f64::MAX;
    let mut max_y = f64::MIN;

    if autoscale {
        let mut valid_points = 0;
        for ds in state.datasets.iter() {
            if !ds.visible { continue; }
            for s in &ds.points {
                let (cx, cy) = get_coords(s);
                if cx.is_nan() || cy.is_nan() { continue; }

                if cx < min_x { min_x = cx; }
                if cx > max_x { max_x = cx; }
                if cy < min_y { min_y = cy; }
                if cy > max_y { max_y = cy; }
                valid_points += 1;
            }
        }

        if valid_points == 0 {
            if show_dome {
                let (mut def_x_min, mut def_x_max) = match current_plot {
                    PlotType::PT => (0.0, 25.0),
                    PlotType::RhoT => (0.0, 1100.0),
                    PlotType::VT => (0.0005, 0.2),
                };
                let (mut def_y_min, mut def_y_max) = (270.0, 660.0);

                if swap_axes {
                    std::mem::swap(&mut def_x_min, &mut def_y_min);
                    std::mem::swap(&mut def_x_max, &mut def_y_max);
                }

                min_x = def_x_min; max_x = def_x_max;
                min_y = def_y_min; max_y = def_y_max;
            } else {
                min_x = 0.0; max_x = 100.0;
                min_y = 273.15; max_y = 2273.15;
                if swap_axes { std::mem::swap(&mut min_x, &mut min_y); std::mem::swap(&mut max_x, &mut max_y); }
            }
        } else {
            if max_x <= min_x {
                let pad = if max_x == 0.0 { 1.0 } else { max_x.abs() * 0.2 + 1.0 };
                min_x -= pad; max_x += pad;
            } else {
                let pad_x = (max_x - min_x) * 0.1;
                min_x -= pad_x; max_x += pad_x;
            }

            if max_y <= min_y {
                let pad = if max_y == 0.0 { 10.0 } else { max_y.abs() * 0.2 + 10.0 };
                min_y -= pad; max_y += pad;
            } else {
                let pad_y = (max_y - min_y) * 0.1;
                min_y -= pad_y; max_y += pad_y;
            }
        }
    } else {
        let (v_min, v_max, t_min, t_max) = custom_limits;
        if swap_axes {
            min_x = t_min; max_x = t_max;
            min_y = v_min; max_y = v_max;
        } else {
            min_x = v_min; max_x = v_max;
            min_y = t_min; max_y = t_max;
        }
        if min_x >= max_x { max_x = min_x + 1.0; }
        if min_y >= max_y { max_y = min_y + 1.0; }
    }

    if let Ok(mut chart) = ChartBuilder::on(root)
        .margin(40)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(min_x..max_x, min_y..max_y)
    {
        let desc_val = match current_plot {
            PlotType::PT => "Давление (p), МПа",
            PlotType::RhoT => "Плотность (rho), кг/м3",
            PlotType::VT => "Уд. объем (v), м3/кг",
        };
        let desc_t = "Температура (T), К";

        let (x_desc, y_desc) = if swap_axes { (desc_t, desc_val) } else { (desc_val, desc_t) };

        chart.configure_mesh().x_desc(x_desc).y_desc(y_desc).draw().ok();

        if show_dome {
            let sat_style = ShapeStyle::from(&bright_purple).stroke_width(2);

            if current_plot == PlotType::PT {
                chart.draw_series(LineSeries::new(sat_liq.iter().map(|s| get_coords(s)), sat_style)).ok();
            } else {
                let mut dome_points: Vec<(f64, f64)> = sat_liq.iter().map(|s| get_coords(s)).collect();
                let mut vap_points: Vec<(f64, f64)> = sat_vap.iter().map(|s| get_coords(s)).collect();
                vap_points.reverse();
                dome_points.extend(vap_points);

                chart.draw_series(LineSeries::new(dome_points, sat_style)).ok();
            }
        }

        let mut color_idx = 0;
        for ds in state.datasets.iter() {
            if !ds.visible || ds.points.is_empty() { continue; }

            let color = get_palette_color(color_idx);
            color_idx += 1;

            chart.draw_series(ds.points.iter().filter_map(|s| {
                let (cx, cy) = get_coords(s);
                if cx.is_nan() || cy.is_nan() { return None; }
                if cx < min_x || cx > max_x || cy < min_y || cy > max_y { return None; }
                Some(Circle::new((cx, cy), 4, color.filled()))
            }))
                .unwrap()
                .label(&ds.name)
                .legend(move |(x, y)| Circle::new((x, y), 4, color.filled()));
        }

        chart.configure_series_labels()
            .position(SeriesLabelPosition::UpperRight)
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw().ok();
    }
}