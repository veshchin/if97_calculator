// desktop_fltk/src/main.rs

#![windows_subsystem = "windows"]

use fltk::{
    app,
    browser::{HoldBrowser, CheckBrowser},
    button::{Button, CheckButton},
    dialog,
    enums::{Align, CallbackTrigger, Color as FltkColor, Font, FrameType},
    frame::Frame,
    group::{Flex, Group, Tabs},
    input::{Input, MultilineInput},
    menu::Choice,
    prelude::*,
    window::Window,
};
use plotters::prelude::*;
use std::sync::{Arc, Mutex};
use std::fs;

use if97_core::domain::calculator::Calculator;
use if97_core::app::router::Router;
use if97_core::domain::state::WaterState;

#[derive(Clone, Copy, PartialEq)]
enum PlotType {
    PT,
    RhoT,
    VT,
}

struct SavedData {
    name: String,
    points: Vec<WaterState>,
    visible: bool,
}

fn get_palette_color(idx: usize) -> RGBColor {
    let palette = [RED, BLUE, GREEN, MAGENTA, CYAN, BLACK];
    palette[idx % palette.len()]
}

fn main() {
    let app = app::App::default();

    let shared_datasets = Arc::new(Mutex::new(vec![
        SavedData {
            name: "Текущая таблица".to_string(),
            points: vec![],
            visible: true
        }
    ]));

    let last_single_result = Arc::new(Mutex::new(None::<WaterState>));
    let shared_plot_type = Arc::new(Mutex::new(PlotType::PT));
    let shared_show_dome = Arc::new(Mutex::new(true));
    let shared_swap_axes = Arc::new(Mutex::new(false));

    // Новые состояния для управления масштабом
    let shared_autoscale = Arc::new(Mutex::new(true));
    // Масштаб теперь хранится как (val_min, val_max, t_min, t_max)
    let shared_custom_limits = Arc::new(Mutex::new((0.0, 100.0, 273.15, 1000.0)));

    let mut wind = Window::default().with_size(1050, 700).with_label("IAPWS-IF97 Calculator Pro");
    wind.make_resizable(true);

    let main_group = Group::default_fill();
    let tabs = Tabs::new(10, 10, 1030, 680, "");

    // ==========================================
    // UI: ВКЛАДКА 1 (ОДИНОЧНЫЙ РАСЧЕТ)
    // ==========================================
    let grp_single = Group::new(10, 35, 1030, 655, " Одиночный расчет ");

    let mut instruction_single = Frame::new(20, 45, 1010, 40, "Инструкция: Выберите режим расчета и введите параметры. \nВнимание: Дробная часть числа должна отделяться ТОЧКОЙ (например, 14.5, а не 14,5).");
    instruction_single.set_align(Align::Left | Align::Inside);
    instruction_single.set_label_color(FltkColor::Dark3);
    instruction_single.set_label_font(Font::HelveticaItalic);

    let mut flex_single = Flex::new(20, 90, 400, 300, "").column();
    flex_single.set_pad(15);

    let mut choice_mode = Choice::default().with_label("Что известно:");
    choice_mode.set_align(Align::TopLeft);
    choice_mode.add_choice("Давление и Температура (p, T)|Давление и Плотность (p, rho)|Давление и Энтальпия (p, h)|Давление и Энтропия (p, s)|Линия насыщения (p, x)");
    choice_mode.set_value(0);

    let mut input_a = Input::default().with_label("Давление (p), МПа:");
    input_a.set_align(Align::TopLeft);
    let mut input_b = Input::default().with_label("Температура (T), К:");
    input_b.set_align(Align::TopLeft);

    let mut btn_row_single = Flex::default().row();
    let mut btn_calc = Button::default().with_label("Рассчитать");
    btn_calc.set_color(FltkColor::from_rgb(0, 120, 215));
    btn_calc.set_label_color(FltkColor::White);

    let mut btn_save_single = Button::default().with_label("Сохранить точку");
    btn_save_single.set_color(FltkColor::from_rgb(34, 139, 34));
    btn_save_single.set_label_color(FltkColor::White);
    btn_row_single.end();

    let mut res_frame = Frame::default().with_label("Ожидание данных...");
    res_frame.set_frame(FrameType::DownBox);
    res_frame.set_color(FltkColor::White);

    flex_single.end();
    grp_single.end();

    // ==========================================
    // UI: ВКЛАДКА 2 (ТАБЛИЧНЫЙ РАСЧЕТ)
    // ==========================================
    let grp_batch = Group::new(10, 35, 1030, 655, " Табличный расчет ");

    let mut instruction_batch = Frame::new(20, 45, 1010, 40, "Инструкция: Вставьте данные в левое поле или загрузите из файла (.csv, .txt). \nРазделители столбцов: пробел, табуляция, запятая или точка с запятой. Дробная часть — ТОЧКА.");
    instruction_batch.set_align(Align::Left | Align::Inside);
    instruction_batch.set_label_color(FltkColor::Dark3);
    instruction_batch.set_label_font(Font::HelveticaItalic);

    let mut btn_load = Button::new(20, 85, 120, 30, "Загрузить файл");

    let mut btn_save_batch = Button::new(150, 85, 150, 30, "Сохранить таблицу");
    btn_save_batch.set_color(FltkColor::from_rgb(34, 139, 34));
    btn_save_batch.set_label_color(FltkColor::White);

    let mut choice_batch_mode = Choice::new(310, 85, 200, 30, "");
    choice_batch_mode.add_choice("p, T|rho, T|p, h|p, s|p, x");
    choice_batch_mode.set_value(0);

    let mut flex_batch = Flex::new(20, 125, 1000, 520, "").row();
    flex_batch.set_pad(20);

    let mut input_area = MultilineInput::default().with_label("Ввод данных (2 колонки):");
    input_area.set_align(Align::TopLeft);

    let mut output_table = HoldBrowser::default().with_label("Результат:");
    output_table.set_align(Align::TopLeft);
    output_table.set_text_size(14);
    output_table.set_column_widths(&[60, 60, 55, 120, 80, 80, 80, 80, 80, 80, 80, 60, 0]);
    output_table.set_column_char('\t');

    flex_batch.fixed(&input_area, 180);
    flex_batch.end();
    grp_batch.end();

    // ==========================================
    // UI: ВКЛАДКА 3 (ГРАФИКИ)
    // ==========================================
    let grp_plot = Group::new(10, 35, 1030, 655, " Графики ");

    // Строка 1: Основные настройки графиков
    let mut choice_plot_type = Choice::new(20, 45, 180, 30, "Тип диаграммы:");
    choice_plot_type.set_align(Align::TopLeft);
    choice_plot_type.add_choice("p-T Диаграмма|rho-T Диаграмма|v-T Диаграмма");
    choice_plot_type.set_value(0);

    let mut btn_select_data = Button::new(210, 45, 140, 30, "Выбрать данные");

    let mut check_dome = CheckButton::new(360, 45, 180, 30, "Показывать купол");
    check_dome.set_value(true);

    let mut check_swap = CheckButton::new(550, 45, 130, 30, "Поменять оси");
    check_swap.set_value(false);

    let mut btn_export_plot = Button::new(690, 45, 150, 30, "Экспорт в PNG");
    btn_export_plot.set_color(FltkColor::from_rgb(100, 149, 237));
    btn_export_plot.set_label_color(FltkColor::White);

    // Строка 2: Настройки масштаба (Zoom) по параметрам
    let mut check_autoscale = CheckButton::new(20, 85, 120, 30, "Автомасштаб");
    check_autoscale.set_value(true);

    let mut inp_val_min = Input::new(250, 85, 60, 30, "p, МПа от:");
    let mut inp_val_max = Input::new(350, 85, 60, 30, "до:");
    let mut inp_t_min = Input::new(480, 85, 60, 30, "T, К от:");
    let mut inp_t_max = Input::new(570, 85, 60, 30, "до:");

    inp_val_min.set_value("0.0");
    inp_val_max.set_value("100.0");
    inp_t_min.set_value("273.15");
    inp_t_max.set_value("1000.0");

    inp_val_min.deactivate();
    inp_val_max.deactivate();
    inp_t_min.deactivate();
    inp_t_max.deactivate();

    let mut btn_apply_limits = Button::new(650, 85, 150, 30, "Применить масштаб");
    btn_apply_limits.deactivate();

    let mut plot_frame = Frame::new(20, 130, 1000, 515, "");
    plot_frame.set_color(FltkColor::White);
    plot_frame.set_frame(FrameType::FlatBox);

    grp_plot.end();
    tabs.end();
    main_group.end();
    wind.end();
    wind.show();

    // ==========================================
    // ЛОГИКА И КОЛБЕКИ
    // ==========================================

    choice_mode.set_callback({
        let mut ia = input_a.clone();
        let mut ib = input_b.clone();
        move |c| {
            match c.value() {
                0 => { ia.set_label("Давление (p), МПа:"); ib.set_label("Температура (T), К:"); }
                1 => { ia.set_label("Давление (p), МПа:"); ib.set_label("Плотность (rho), кг/м3:"); }
                2 => { ia.set_label("Давление (p), МПа:"); ib.set_label("Энтальпия (h), кДж/кг:"); }
                3 => { ia.set_label("Давление (p), МПа:"); ib.set_label("Энтропия (s), кДж/(кг*К):"); }
                4 => { ia.set_label("Давление (p), МПа:"); ib.set_label("Степень сухости (x), 0.0-1.0:"); }
                _ => {}
            }
            ia.redraw();
            ib.redraw();
        }
    });

    btn_calc.set_callback({
        let mut res = res_frame.clone();
        let i_a = input_a.clone();
        let i_b = input_b.clone();
        let c_m = choice_mode.clone();
        let lsr = last_single_result.clone();
        move |_| {
            let val_a = i_a.value().replace(',', ".").parse::<f64>().unwrap_or(f64::NAN);
            let val_b = i_b.value().replace(',', ".").parse::<f64>().unwrap_or(f64::NAN);

            if val_a.is_nan() || val_b.is_nan() {
                res.set_label("Ошибка: Введите корректные числовые значения.");
                res.set_label_color(FltkColor::Red);
                *lsr.lock().unwrap() = None;
                return;
            }

            let result = match c_m.value() {
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
                        "РЕЗУЛЬТАТ:\n\
                        Регион: {:?}\n\
                        p = {:.6} МПа\n\
                        T = {:.2} К\n\
                        v = {:.6} м3/кг\n\
                        rho = {:.2} кг/м3\n\
                        h = {:.2} кДж/кг\n\
                        s = {:.4} кДж/(кг·К)\n\
                        u = {:.2} кДж/кг\n\
                        cp = {:.4} кДж/(кг·К)\n\
                        w = {:.2} м/с",
                        s.region, s.p, s.t, s.v, s.rho, s.h, s.s, u, s.cp, s.w
                    );
                    res.set_label(&out);
                    res.set_label_color(FltkColor::Black);
                    *lsr.lock().unwrap() = Some(s);
                }
                Err(e) => {
                    res.set_label(&format!("Ошибка расчета:\n{}", e));
                    res.set_label_color(FltkColor::Red);
                    *lsr.lock().unwrap() = None;
                }
            }
        }
    });

    btn_save_single.set_callback({
        let lsr = last_single_result.clone();
        let sds = shared_datasets.clone();
        let mut pf = plot_frame.clone();
        move |_| {
            if let Some(state) = &*lsr.lock().unwrap() {
                if let Some(name) = dialog::input(150, 200, "Введите имя для точки:", "Точка 1") {
                    if !name.is_empty() {
                        sds.lock().unwrap().push(SavedData {
                            name,
                            points: vec![state.clone()],
                            visible: true,
                        });
                        dialog::message(150, 200, "Точка успешно сохранена и добавлена на график!");
                        if let Some(mut win) = pf.window() { win.redraw(); } else { pf.redraw(); }
                    }
                }
            } else {
                dialog::alert(150, 200, "Сначала выполните успешный расчет!");
            }
        }
    });

    btn_load.set_callback({
        let mut ia = input_area.clone();
        move |_| {
            let mut chooser = dialog::FileChooser::new(".", "*.{csv,txt}", dialog::FileChooserType::Single, "Выберите файл с данными");
            chooser.show();
            while chooser.shown() { app::wait(); }
            if let Some(filename) = chooser.value(1) {
                if let Ok(content) = fs::read_to_string(filename) {
                    ia.set_value(&content);
                    ia.do_callback();
                }
            }
        }
    });

    btn_save_batch.set_callback({
        let sds = shared_datasets.clone();
        let mut pf = plot_frame.clone();
        move |_| {
            let current_points = sds.lock().unwrap()[0].points.clone();
            if current_points.is_empty() {
                dialog::alert(150, 200, "Текущая таблица пуста!");
                return;
            }
            if let Some(name) = dialog::input(150, 200, "Введите имя для набора данных:", "Набор 1") {
                if !name.is_empty() {
                    sds.lock().unwrap().push(SavedData {
                        name,
                        points: current_points,
                        visible: true,
                    });
                    dialog::message(150, 200, "Набор успешно сохранен!");
                    if let Some(mut win) = pf.window() { win.redraw(); } else { pf.redraw(); }
                }
            }
        }
    });

    input_area.set_callback({
        let mut out_table = output_table.clone();
        let sds = shared_datasets.clone();
        let c_m = choice_batch_mode.clone();
        let mut pf = plot_frame.clone();
        move |i| {
            let mut new_points = Vec::new();
            let mode = c_m.value();

            out_table.clear();
            out_table.add("P_MPa\tT_K\tRegion\tx_vapor_fraction\tv_m3_kg\trho_kg_m3\th_kJ_kg\ts_kJ_kgK\tu_kJ_kg\tcp_kJ_kgK\tcv_kJ_kgK\tw_m_s");

            for line in i.value().lines() {
                if line.trim().is_empty() { continue; }

                let cleaned_line = line.replace(';', " ").replace(',', " ");
                let parts: Vec<&str> = cleaned_line.split_whitespace().collect();

                if parts.len() >= 2 {
                    if let (Ok(val_a), Ok(val_b)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {

                        let result = match mode {
                            0 => Calculator::calculate_pt(val_a, val_b),
                            1 => Calculator::calculate_rhot(val_a, val_b),
                            2 => Router::calculate_ph(val_a, val_b),
                            3 => Router::calculate_ps(val_a, val_b),
                            4 => Calculator::calculate_px(val_a, val_b),
                            _ => Err("Unknown mode"),
                        };

                        match result {
                            Ok(s) => {
                                let region_num = match s.region {
                                    if97_core::domain::state::Region::Region1 => "1",
                                    if97_core::domain::state::Region::Region2 => "2",
                                    if97_core::domain::state::Region::Region3 => "3",
                                    if97_core::domain::state::Region::Region4 => "4",
                                    if97_core::domain::state::Region::Region5 => "5",
                                    _ => "-",
                                };

                                let x_str = if mode == 4 { format!("{:.4}", val_b) } else { "-".to_string() };
                                let u = s.h - s.p * s.v * 1000.0;

                                let format_val = |v: f64, prec: usize| -> String {
                                    if v.is_nan() { "-".to_string() } else { format!("{:.*}", prec, v) }
                                };

                                let row_str = format!(
                                    "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t-\t{}",
                                    format_val(s.p, 5), format_val(s.t, 2), region_num, x_str,
                                    format_val(s.v, 6), format_val(s.rho, 2), format_val(s.h, 4),
                                    format_val(s.s, 4), format_val(u, 4), format_val(s.cp, 4),
                                    format_val(s.w, 2)
                                );
                                out_table.add(&row_str);
                                new_points.push(s);
                            }
                            Err(_) => {
                                out_table.add("Ошибка\t-\t-\t-\t-\t-\t-\t-\t-\t-\t-\t-");
                            }
                        }
                    }
                }
            }
            if let Ok(mut datasets) = sds.lock() {
                datasets[0].points = new_points;
            }

            if let Some(mut win) = pf.window() { win.redraw(); } else { pf.redraw(); }
        }
    });
    input_area.set_trigger(CallbackTrigger::Changed);

    choice_plot_type.set_callback({
        let pt = shared_plot_type.clone();
        let mut pf = plot_frame.clone();
        let mut ivm = inp_val_min.clone();
        move |c| {
            if let Ok(mut p_type) = pt.lock() {
                *p_type = match c.value() {
                    0 => { ivm.set_label("p, МПа от:"); PlotType::PT },
                    1 => { ivm.set_label("rho, кг/м3 от:"); PlotType::RhoT },
                    _ => { ivm.set_label("v, м3/кг от:"); PlotType::VT },
                };
            }
            if let Some(mut win) = pf.window() { win.redraw(); } else { pf.redraw(); }
        }
    });

    check_dome.set_callback({
        let sd = shared_show_dome.clone();
        let mut pf = plot_frame.clone();
        move |c| {
            if let Ok(mut val) = sd.lock() { *val = c.value(); }
            if let Some(mut win) = pf.window() { win.redraw(); } else { pf.redraw(); }
        }
    });

    check_swap.set_callback({
        let sa = shared_swap_axes.clone();
        let mut pf = plot_frame.clone();
        move |c| {
            if let Ok(mut val) = sa.lock() { *val = c.value(); }
            if let Some(mut win) = pf.window() { win.redraw(); } else { pf.redraw(); }
        }
    });

    // Управление галочкой Автомасштаб
    check_autoscale.set_callback({
        let sauto = shared_autoscale.clone();
        let mut i_v_min = inp_val_min.clone();
        let mut i_v_max = inp_val_max.clone();
        let mut i_t_min = inp_t_min.clone();
        let mut i_t_max = inp_t_max.clone();
        let mut btn_apply = btn_apply_limits.clone();
        let mut pf = plot_frame.clone();
        move |c| {
            if let Ok(mut val) = sauto.lock() {
                *val = c.value();
                if *val {
                    i_v_min.deactivate(); i_v_max.deactivate();
                    i_t_min.deactivate(); i_t_max.deactivate();
                    btn_apply.deactivate();
                } else {
                    i_v_min.activate(); i_v_max.activate();
                    i_t_min.activate(); i_t_max.activate();
                    btn_apply.activate();
                }
            }
            if let Some(mut win) = pf.window() { win.redraw(); } else { pf.redraw(); }
        }
    });

    // Обработка кнопки "Применить масштаб"
    btn_apply_limits.set_callback({
        let i_v_min = inp_val_min.clone();
        let i_v_max = inp_val_max.clone();
        let i_t_min = inp_t_min.clone();
        let i_t_max = inp_t_max.clone();
        let s_limits = shared_custom_limits.clone();
        let mut pf = plot_frame.clone();
        move |_| {
            let v_min = i_v_min.value().replace(',', ".").parse::<f64>().unwrap_or(0.0);
            let v_max = i_v_max.value().replace(',', ".").parse::<f64>().unwrap_or(100.0);
            let t_min = i_t_min.value().replace(',', ".").parse::<f64>().unwrap_or(273.15);
            let t_max = i_t_max.value().replace(',', ".").parse::<f64>().unwrap_or(1000.0);

            if let Ok(mut limits) = s_limits.lock() {
                *limits = (v_min, v_max, t_min, t_max);
            }
            if let Some(mut win) = pf.window() { win.redraw(); } else { pf.redraw(); }
        }
    });

    btn_select_data.set_callback({
        let sds = shared_datasets.clone();
        let mut pf = plot_frame.clone();
        move |_| {
            let mut win = Window::default().with_size(300, 400).with_label("Выбор данных для отрисовки");
            let mut cb = CheckBrowser::default().with_size(280, 320).with_pos(10, 10);
            let mut btn_apply = Button::default().with_size(100, 40).with_label("Применить").with_pos(100, 340);

            {
                let datasets = sds.lock().unwrap();
                for ds in datasets.iter() {
                    cb.add(&ds.name, ds.visible);
                }
            }

            win.make_modal(true);
            win.show();

            btn_apply.set_callback({
                let sds = sds.clone();
                let mut cb = cb.clone();
                let mut win = win.clone();
                let mut pf = pf.clone();
                move |_| {
                    let mut datasets = sds.lock().unwrap();
                    for i in 0..datasets.len() {
                        datasets[i].visible = cb.checked((i + 1) as i32);
                    }
                    win.hide();
                    if let Some(mut w) = pf.window() { w.redraw(); } else { pf.redraw(); }
                }
            });

            while win.shown() { app::wait(); }
        }
    });

    btn_export_plot.set_callback({
        let sds = shared_datasets.clone();
        let pt_type = shared_plot_type.clone();
        let sd_state = shared_show_dome.clone();
        let sa_state = shared_swap_axes.clone();
        let saut_state = shared_autoscale.clone();
        let slim_state = shared_custom_limits.clone();
        move |_| {
            let mut chooser = dialog::FileChooser::new(".", "*.png", dialog::FileChooserType::Create, "Сохранить график как...");
            chooser.show();
            while chooser.shown() { app::wait(); }
            if let Some(mut filename) = chooser.value(1) {
                if !filename.ends_with(".png") {
                    filename.push_str(".png");
                }

                let fw = 1920;
                let fh = 1080;
                let root = BitMapBackend::new(&filename, (fw, fh)).into_drawing_area();
                root.fill(&WHITE).ok();

                let current_plot = *pt_type.lock().unwrap();
                let datasets = sds.lock().unwrap();
                let show_dome = *sd_state.lock().unwrap();
                let swap_axes = *sa_state.lock().unwrap();
                let autoscale = *saut_state.lock().unwrap();
                let custom_limits = *slim_state.lock().unwrap();

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
                    for ds in datasets.iter() {
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

                if let Ok(mut chart) = ChartBuilder::on(&root)
                    .margin(40)
                    .x_label_area_size(50)
                    .y_label_area_size(70)
                    .build_cartesian_2d(min_x..max_x, min_y..max_y)
                {
                    let desc_val = match current_plot {
                        PlotType::PT => "Давление (p), МПа",
                        PlotType::RhoT => "Плотность (rho), кг/м3",
                        PlotType::VT => "Уд. объем (v), м3/кг",
                    };
                    let desc_t = "Температура (T), К";

                    let (x_desc, y_desc) = if swap_axes { (desc_t, desc_val) } else { (desc_val, desc_t) };

                    chart.configure_mesh()
                        .x_desc(x_desc)
                        .y_desc(y_desc)
                        .label_style(("sans-serif", 20).into_font())
                        .draw().ok();

                    if show_dome {
                        let sat_style = ShapeStyle::from(&bright_purple).stroke_width(3);

                        if current_plot == PlotType::PT {
                            chart.draw_series(LineSeries::new(
                                sat_liq.iter().map(|s| get_coords(s)),
                                sat_style,
                            )).ok();
                        } else {
                            let mut dome_points: Vec<(f64, f64)> = sat_liq.iter().map(|s| get_coords(s)).collect();
                            let mut vap_points: Vec<(f64, f64)> = sat_vap.iter().map(|s| get_coords(s)).collect();
                            vap_points.reverse();
                            dome_points.extend(vap_points);

                            chart.draw_series(LineSeries::new(
                                dome_points,
                                sat_style,
                            )).ok();
                        }
                    }

                    let mut color_idx = 0;
                    for ds in datasets.iter() {
                        if !ds.visible || ds.points.is_empty() { continue; }

                        let color = get_palette_color(color_idx);
                        color_idx += 1;

                        chart.draw_series(ds.points.iter().filter_map(|s| {
                            let (cx, cy) = get_coords(s);
                            if cx.is_nan() || cy.is_nan() { return None; }
                            // Отсекаем точки вне масштаба
                            if cx < min_x || cx > max_x || cy < min_y || cy > max_y { return None; }
                            Some(Circle::new((cx, cy), 6, color.filled()))
                        }))
                            .unwrap()
                            .label(&ds.name)
                            .legend(move |(x, y)| Circle::new((x, y), 6, color.filled()));
                    }

                    chart.configure_series_labels()
                        .position(SeriesLabelPosition::UpperRight)
                        .background_style(&WHITE.mix(0.8))
                        .border_style(&BLACK)
                        .label_font(("sans-serif", 24))
                        .draw().ok();
                }

                root.present().ok();
                dialog::message(150, 200, "График успешно сохранен!");
            }
        }
    });

    plot_frame.draw({
        let sds = shared_datasets.clone();
        let pt_type = shared_plot_type.clone();
        let sd_state = shared_show_dome.clone();
        let sa_state = shared_swap_axes.clone();
        let saut_state = shared_autoscale.clone();
        let slim_state = shared_custom_limits.clone();

        move |f| {
            let fw = f.w();
            let fh = f.h();
            if fw == 0 || fh == 0 { return; }

            let mut buffer = vec![0u8; (fw * fh * 3) as usize];
            {
                let root = BitMapBackend::with_buffer(&mut buffer, (fw as u32, fh as u32)).into_drawing_area();
                root.fill(&WHITE).ok();

                let current_plot = *pt_type.lock().unwrap();
                let datasets = sds.lock().unwrap();
                let show_dome = *sd_state.lock().unwrap();
                let swap_axes = *sa_state.lock().unwrap();
                let autoscale = *saut_state.lock().unwrap();
                let custom_limits = *slim_state.lock().unwrap();

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
                    for ds in datasets.iter() {
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

                if let Ok(mut chart) = ChartBuilder::on(&root)
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

                    chart.configure_mesh()
                        .x_desc(x_desc)
                        .y_desc(y_desc)
                        .draw().ok();

                    if show_dome {
                        let sat_style = ShapeStyle::from(&bright_purple).stroke_width(2);

                        if current_plot == PlotType::PT {
                            chart.draw_series(LineSeries::new(
                                sat_liq.iter().map(|s| get_coords(s)),
                                sat_style,
                            )).ok();
                        } else {
                            let mut dome_points: Vec<(f64, f64)> = sat_liq.iter().map(|s| get_coords(s)).collect();
                            let mut vap_points: Vec<(f64, f64)> = sat_vap.iter().map(|s| get_coords(s)).collect();
                            vap_points.reverse();
                            dome_points.extend(vap_points);

                            chart.draw_series(LineSeries::new(
                                dome_points,
                                sat_style,
                            )).ok();
                        }
                    }

                    let mut color_idx = 0;
                    for ds in datasets.iter() {
                        if !ds.visible || ds.points.is_empty() { continue; }

                        let color = get_palette_color(color_idx);
                        color_idx += 1;

                        chart.draw_series(ds.points.iter().filter_map(|s| {
                            let (cx, cy) = get_coords(s);
                            if cx.is_nan() || cy.is_nan() { return None; }
                            // Отсекаем точки вне масштаба
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

            fltk::draw::draw_image(&buffer, f.x(), f.y(), fw, fh, fltk::enums::ColorDepth::Rgb8).ok();
        }
    });

    app.run().unwrap();
}