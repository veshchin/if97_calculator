// desktop_fltk/src/main.rs

#![windows_subsystem = "windows"]

mod state;
mod plot;
mod ui;

use fltk::{app, dialog, prelude::*, window::Window, browser::CheckBrowser, button::Button};
use std::fs;

use if97_core::domain::calculator::Calculator;
use if97_core::app::router::Router;

use state::{AppState, Message, SavedData};
use ui::MainUI;
use plot::renderer::render_plot_to_file;

fn main() {
    let fltk_app = app::App::default();
    let (sender, receiver) = app::channel::<Message>();

    let mut state = AppState::new();
    let mut main_ui = MainUI::new(sender.clone());

    main_ui.window.show();
    main_ui.plot_tab.redraw_plot(&state);

    while fltk_app.wait() {
        if let Some(msg) = receiver.recv() {
            match msg {
                // --- Логика одиночного расчета ---
                Message::CalculateSingle { mode, val_a, val_b } => {
                    if val_a.is_nan() || val_b.is_nan() {
                        main_ui.single_tab.update_result("Ошибка: Введите корректные числовые значения.", true);
                        state.last_single_result = None;
                        continue;
                    }

                    let result = match mode {
                        0 => Calculator::calculate_pt(val_a, val_b),
                        1 => Calculator::calculate_rhot(val_a, val_b),
                        2 => Router::calculate_ph(val_a, val_b),
                        3 => Router::calculate_ps(val_a, val_b),
                        4 => Calculator::calculate_px(val_a, val_b),
                        _ => Err("Неверный режим"),
                    };

                    match result {
                        Ok(s) => {
                            let u = s.h - s.p * s.v * 1000.0;
                            let out = format!(
                                "РЕЗУЛЬТАТ:\nРегион: {:?}\np = {:.6} МПа\nT = {:.2} К\nv = {:.6} м3/кг\nrho = {:.2} кг/м3\nh = {:.2} кДж/кг\ns = {:.4} кДж/(кг·К)\nu = {:.2} кДж/кг\ncp = {:.4} кДж/(кг·К)\nw = {:.2} м/с",
                                s.region, s.p, s.t, s.v, s.rho, s.h, s.s, u, s.cp, s.w
                            );
                            main_ui.single_tab.update_result(&out, false);
                            state.last_single_result = Some(s);
                        }
                        Err(e) => {
                            main_ui.single_tab.update_result(&format!("Ошибка расчета:\n{}", e), true);
                            state.last_single_result = None;
                        }
                    }
                }
                Message::SaveSinglePoint => {
                    if let Some(st) = &state.last_single_result {
                        if let Some(name) = dialog::input(150, 200, "Введите имя для точки:", "Точка 1") {
                            if !name.is_empty() {
                                state.datasets.push(SavedData { name, points: vec![st.clone()], visible: true });
                                dialog::message(150, 200, "Точка успешно сохранена!");
                                main_ui.plot_tab.redraw_plot(&state);
                            }
                        }
                    } else {
                        dialog::alert(150, 200, "Сначала выполните успешный расчет!");
                    }
                }

                // --- Логика табличного расчета ---
                Message::LoadBatchFile => {
                    let mut chooser = dialog::FileChooser::new(".", "*.{csv,txt}", dialog::FileChooserType::Single, "Выберите файл с данными");
                    chooser.show();
                    while chooser.shown() { app::wait(); }
                    if let Some(filename) = chooser.value(1) {
                        if let Ok(content) = fs::read_to_string(filename) {
                            main_ui.batch_tab.set_input(&content);
                        }
                    }
                }
                Message::SaveBatchTable => {
                    let current_points = state.datasets[0].points.clone();
                    if current_points.is_empty() {
                        dialog::alert(150, 200, "Текущая таблица пуста!");
                    } else if let Some(name) = dialog::input(150, 200, "Введите имя для набора данных:", "Набор 1") {
                        if !name.is_empty() {
                            state.datasets.push(SavedData { name, points: current_points, visible: true });
                            dialog::message(150, 200, "Набор успешно сохранен!");
                            main_ui.plot_tab.redraw_plot(&state);
                        }
                    }
                }
                Message::BatchDataChanged { mode, content } => {
                    let mut new_points = Vec::new();
                    main_ui.batch_tab.output_table.clear();
                    main_ui.batch_tab.output_table.add("P_MPa\tT_K\tRegion\tx_vapor_fraction\tv_m3_kg\trho_kg_m3\th_kJ_kg\ts_kJ_kgK\tu_kJ_kg\tcp_kJ_kgK\tcv_kJ_kgK\tw_m_s");

                    for line in content.lines() {
                        if line.trim().is_empty() { continue; }
                        let cleaned = line.replace(';', " ").replace(',', " ");
                        let parts: Vec<&str> = cleaned.split_whitespace().collect();

                        if parts.len() >= 2 {
                            if let (Ok(va), Ok(vb)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
                                let res = match mode {
                                    0 => Calculator::calculate_pt(va, vb),
                                    1 => Calculator::calculate_rhot(va, vb),
                                    2 => Router::calculate_ph(va, vb),
                                    3 => Router::calculate_ps(va, vb),
                                    4 => Calculator::calculate_px(va, vb),
                                    _ => Err("Err"),
                                };
                                match res {
                                    Ok(s) => {
                                        let rn = match s.region {
                                            if97_core::domain::state::Region::Region1 => "1",
                                            if97_core::domain::state::Region::Region2 => "2",
                                            if97_core::domain::state::Region::Region3 => "3",
                                            if97_core::domain::state::Region::Region4 => "4",
                                            if97_core::domain::state::Region::Region5 => "5",
                                            _ => "-",
                                        };
                                        let xs = if mode == 4 { format!("{:.4}", vb) } else { "-".to_string() };
                                        let u = s.h - s.p * s.v * 1000.0;
                                        let fmt = |v: f64, p: usize| if v.is_nan() { "-".into() } else { format!("{:.*}", p, v) };

                                        main_ui.batch_tab.output_table.add(&format!(
                                            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t-\t{}",
                                            fmt(s.p, 5), fmt(s.t, 2), rn, xs, fmt(s.v, 6), fmt(s.rho, 2), fmt(s.h, 4), fmt(s.s, 4), fmt(u, 4), fmt(s.cp, 4), fmt(s.w, 2)
                                        ));
                                        new_points.push(s);
                                    }
                                    Err(_) => main_ui.batch_tab.output_table.add("Ошибка\t-\t-\t-\t-\t-\t-\t-\t-\t-\t-\t-"),
                                }
                            }
                        }
                    }
                    state.datasets[0].points = new_points;
                    main_ui.plot_tab.redraw_plot(&state);
                }

                // --- Логика графиков ---
                Message::ChangePlotType(pt) => { state.plot_type = pt; main_ui.plot_tab.redraw_plot(&state); }
                Message::ToggleDome(show) => { state.show_dome = show; main_ui.plot_tab.redraw_plot(&state); }
                Message::ToggleSwapAxes(swap) => { state.swap_axes = swap; main_ui.plot_tab.redraw_plot(&state); }
                Message::SetAutoscale(auto) => { state.autoscale = auto; main_ui.plot_tab.redraw_plot(&state); }
                Message::ApplyPlotLimits(vmin, vmax, tmin, tmax) => { state.custom_limits = (vmin, vmax, tmin, tmax); main_ui.plot_tab.redraw_plot(&state); }
                Message::UpdateDataVisibility(vis) => {
                    for (i, v) in vis.into_iter().enumerate() {
                        if i < state.datasets.len() { state.datasets[i].visible = v; }
                    }
                    main_ui.plot_tab.redraw_plot(&state);
                }
                Message::SelectData => {
                    let mut win = Window::default().with_size(300, 400).with_label("Выбор данных");
                    let mut cb = CheckBrowser::default().with_size(280, 320).with_pos(10, 10);
                    let mut btn_apply = Button::default().with_size(100, 40).with_label("Применить").with_pos(100, 340);

                    for ds in &state.datasets { cb.add(&ds.name, ds.visible); }
                    win.make_modal(true);
                    win.show();

                    let s_cloned = sender.clone();
                    btn_apply.set_callback(move |b| {
                        let mut vis = Vec::new();
                        for i in 1..=cb.nitems() { vis.push(cb.checked(i as i32)); }
                        s_cloned.send(Message::UpdateDataVisibility(vis));
                        b.window().unwrap().hide();
                    });
                }
                Message::ExportPlot => {
                    // Формируем путь к рабочему столу текущего пользователя Windows
                    let default_path = std::env::var("USERPROFILE")
                        .map(|p| format!("{}\\Desktop\\IF97_plot.png", p))
                        .unwrap_or_else(|_| "IF97_plot.png".to_string());

                    let mut chooser = dialog::FileChooser::new(
                        &default_path,
                        "*.png",
                        dialog::FileChooserType::Create,
                        "Сохранить как..."
                    );

                    chooser.show();
                    while chooser.shown() { app::wait(); }

                    if let Some(mut filename) = chooser.value(1) {
                        if !filename.ends_with(".png") { filename.push_str(".png"); }

                        if render_plot_to_file(&state, &filename, 1920, 1080).is_ok() {
                            dialog::message(150, 200, "График успешно сохранен!");
                        } else {
                            dialog::alert(150, 200, "Ошибка при сохранении!");
                        }
                    }
                }
            }
        }
    }
}