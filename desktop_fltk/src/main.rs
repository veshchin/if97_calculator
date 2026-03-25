// desktop_fltk/src/main.rs

#![windows_subsystem = "windows"]

use fltk::{
    app, button::Button, enums::{Align, Color as FltkColor, FrameType, CallbackTrigger},
    frame::Frame, group::{Flex, Tabs, Group}, input::{MultilineInput, Input},
    prelude::*, window::Window, menu::Choice, draw,
};

// Импортируем Plotters и явно указываем, что Color — это трейт
use plotters::prelude::*;
use plotters::style::Color as PlottersColorTrait;

use std::sync::{Arc, Mutex};
use if97_core::domain::calculator::Calculator;
use if97_core::app::router::Router;
use if97_core::domain::state::WaterState;

fn main() {
    let app = app::App::default();
    let shared_points = Arc::new(Mutex::new(Vec::<WaterState>::new()));

    let mut wind = Window::default().with_size(900, 650).with_label("IAPWS-IF97 Lab Pro");

    // Главный контейнер
    let main_group = Group::default_fill();

    // 1. Создаем Tabs. Даем им 30px сверху под заголовки
    let mut tabs = Tabs::new(0, 0, 900, 650, "");

    // === ВКЛАДКА 1: ОДИНОЧНЫЙ РАСЧЕТ ===
    // Группа начинается с y=30, чтобы не перекрывать кнопки вкладок
    let grp_single = Group::new(0, 30, 900, 620, " Одиночный расчет ");
    let mut flex_single = Flex::default().with_size(460, 350).center_of(&grp_single).column();
    flex_single.set_pad(10);

    let mut choice_mode = Choice::default().with_label("Режим:");
    choice_mode.add_choice("p, T (Регионы 1,2,3,5)|p, h (Обратное)|p, s (Обратное)|p, x (Двухфазное)");
    choice_mode.set_value(0);

    let input_a = Input::default().with_label("Параметр 1 (p, МПа):");
    let input_b = Input::default().with_label("Параметр 2 (T, h, s, x):");

    let mut btn_calc = Button::default().with_label("Рассчитать");
    btn_calc.set_color(FltkColor::from_rgb(100, 150, 255));
    btn_calc.set_label_color(FltkColor::White);

    let mut res_frame = Frame::default().with_label("Введите данные...");
    res_frame.set_frame(FrameType::DownBox);

    flex_single.end();
    grp_single.end();

    // === ВКЛАДКА 2: ТАБЛИЧНЫЙ ВВОД ===
    let grp_batch = Group::new(0, 30, 900, 620, " Табличный расчет ");
    let mut flex_batch = Flex::default_fill().row();
    flex_batch.set_margin(30);
    flex_batch.set_pad(20);

    let mut input_area = MultilineInput::default().with_label("Ввод (P T):");
    input_area.set_align(Align::TopLeft);

    let mut output_area = MultilineInput::default().with_label("Результат (CSV):");
    output_area.set_align(Align::TopLeft);
    output_area.set_readonly(true);

    flex_batch.fixed(&input_area, 300);
    flex_batch.end();
    grp_batch.end();

    // === ВКЛАДКА 3: ГРАФИКИ ===
    let grp_plot = Group::new(0, 30, 900, 620, " T-s Диаграмма ");
    let mut plot_frame = Frame::default_fill();
    plot_frame.set_color(FltkColor::White);
    plot_frame.set_frame(FrameType::FlatBox);
    grp_plot.end();

    tabs.end();
    main_group.end();
    wind.end();
    wind.show();

    // --- ЛОГИКА ОДИНОЧНОГО РАСЧЕТА ---
    btn_calc.set_callback({
        let mut res = res_frame.clone();
        let i_a = input_a.clone();
        let i_b = input_b.clone();
        let c_m = choice_mode.clone();
        move |_| {
            let p = i_a.value().parse::<f64>().unwrap_or(0.0);
            let b = i_b.value().parse::<f64>().unwrap_or(0.0);
            let result = match c_m.value() {
                0 => Calculator::calculate_pt(p, b),
                1 => Router::calculate_ph(p, b),
                2 => Router::calculate_ps(p, b),
                3 => Calculator::calculate_px(p, b),
                _ => Err("Неверный режим"),
            };
            match result {
                Ok(s) => {
                    res.set_label(&format!("T: {:.2} K | h: {:.2} | s: {:.4}\nRegion: {:?}", s.t, s.h, s.s, s.region));
                    res.set_label_color(FltkColor::Black);
                }
                Err(e) => {
                    res.set_label(&format!("Ошибка:\n{}", e));
                    res.set_label_color(FltkColor::Red);
                }
            }
        }
    });

    // --- ОБНОВЛЕННАЯ ЛОГИКА ТАБЛИЦЫ ---
    input_area.set_callback({
        let mut out = output_area.clone();
        let points = shared_points.clone();
        let mut frame = plot_frame.clone();
        move |i| {
            let mut new_points = Vec::new();
            let mut csv = String::from("p,T,h,s,v\n");

            for line in i.value().lines() {
                // 1. Заменяем все возможные разделители (, ; \t) на пробелы
                let cleaned_line = line.replace(',', " ").replace(';', " ");

                // 2. Теперь спокойно делим по пробелам
                let parts: Vec<&str> = cleaned_line.split_whitespace().collect();

                if parts.len() >= 2 {
                    if let (Ok(p), Ok(t)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
                        if let Ok(s) = Calculator::calculate_pt(p, t) {
                            csv.push_str(&format!("{:.3},{:.2},{:.2},{:.4},{:.6}\n",
                                                  s.p, s.t, s.h, s.s, s.v));
                            new_points.push(s);
                        }
                    }
                }
            }
            out.set_value(&csv);
            if let Ok(mut data) = points.lock() { *data = new_points; }
            frame.redraw();
        }
    });
    input_area.set_trigger(CallbackTrigger::Changed);

    // --- ОТРИСОВКА ГРАФИКА ---
    plot_frame.draw({
        let points = shared_points.clone();
        move |f| {
            let fw = f.w();
            let fh = f.h();
            let mut buffer = vec![0u8; (fw * fh * 3) as usize];
            {
                let root = BitMapBackend::with_buffer(&mut buffer, (fw as u32, fh as u32)).into_drawing_area();
                root.fill(&WHITE).ok();

                let mut chart = ChartBuilder::on(&root)
                    .margin(30).x_label_area_size(40).y_label_area_size(50)
                    .build_cartesian_2d(0.0..9.5, 273.0..900.0).unwrap();

                chart.configure_mesh()
                    .x_desc("Энтропия s, кДж/(кг·К)")
                    .y_desc("Температура T, K")
                    .draw().ok();

                // Пограничная кривая (синяя колоколообразная линия)
                let mut saturation_line = Vec::new();
                for p_bar in (1..220).map(|v| v as f64 * 0.1) {
                    if let Ok(s_liq) = Calculator::calculate_px(p_bar, 0.0) { saturation_line.push((s_liq.s, s_liq.t)); }
                }
                for p_bar in (1..220).rev().map(|v| v as f64 * 0.1) {
                    if let Ok(s_vap) = Calculator::calculate_px(p_bar, 1.0) { saturation_line.push((s_vap.s, s_vap.t)); }
                }
                // Используем микс цвета из трейта Plotters
                chart.draw_series(LineSeries::new(saturation_line, PlottersColorTrait::mix(&BLUE, 0.4))).ok();

                // Точки пользователя (красные)
                if let Ok(data) = points.lock() {
                    chart.draw_series(data.iter().map(|s| {
                        Circle::new((s.s, s.t), 4, RED.filled())
                    })).ok();
                }
            }
            draw::draw_rgb(f, &buffer).ok();
        }
    });

    app.run().unwrap();
}