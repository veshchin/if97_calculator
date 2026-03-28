// desktop_fltk/src/main.rs

#![windows_subsystem = "windows"]

mod state;
mod plot;
mod ui;

use fltk::{app, dialog, prelude::*, window::Window, browser::CheckBrowser, button::Button};
use std::fs;
use std::sync::{Arc, Mutex};
use std::io::Write;
use tracing::{info, error, debug, warn};

// Импорты из обновленного ядра
use if97_core::domain::calculator::If97;
use chrono::Local;

use crate::state::{AppState, Message, SavedData};
use crate::ui::MainUI;
use crate::plot::renderer::render_plot_to_file;

use tracing_subscriber::fmt::writer::MakeWriterExt;

/// Адаптер для захвата логов системы tracing во внутренний буфер приложения
#[derive(Clone)]
struct LogBuffer(Arc<Mutex<String>>);

impl Write for LogBuffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Ok(mut storage) = self.0.lock() {
            storage.push_str(&String::from_utf8_lossy(buf));
        }
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn main() {
    let logs_storage = Arc::new(Mutex::new(String::new()));
    let log_adapter = LogBuffer(logs_storage.clone());

    // 2. Настройка логирования через .and()
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        // std::io::stderr — это функция-замыкание, возвращающая stderr
        // .and() объединяет её с другим замыканием, возвращающим ваш буфер
        .with_writer(std::io::stderr.and(move || log_adapter.clone()))
        .init();

    info!("Приложение запущено. Логи выводятся в консоль и записываются в буфер.");
    // 2. Настройка FLTK и каналов связи
    let fltk_app = app::App::default();
    let (sender, receiver) = app::channel::<Message>();

    let mut state = AppState::new();
    let mut main_ui = MainUI::new(sender.clone());

    main_ui.window.show();
    main_ui.plot_tab.redraw_plot(&state);

    // 3. Главный цикл обработки сообщений
    while fltk_app.wait() {
        if let Some(msg) = receiver.recv() {
            match msg {
                // --- Одиночный расчет ---
                Message::CalculateSingle { mode, val_a, val_b } => {
                    debug!("Запрос на расчет: режим {}, параметры: {}, {}", mode, val_a, val_b);

                    if val_a.is_nan() || val_b.is_nan() {
                        error!("Введены некорректные данные (NaN)");
                        main_ui.single_tab.update_result("Ошибка: Введите числовые значения.", true);
                        state.last_single_result = None;
                        continue;
                    }

                    // Передаем данные в фасад If97, оборачивая f64 в типы размерностей [cite: 651]
                    let result = match mode {
                        0 => If97::pt(val_a.into(), val_b.into()),
                        1 => If97::rhot(val_a.into(), val_b.into()),
                        2 => If97::ph(val_a.into(), val_b.into()),
                        3 => If97::ps(val_a.into(), val_b.into()),
                        4 => If97::px(val_a.into(), val_b.into()),
                        _ => {
                            error!("Неподдерживаемый режим расчета: {}", mode);
                            continue;
                        }
                    };

                    match result {
                        Ok(s) => {
                            info!("Расчет завершен. Регион: {:?}", s.region);

                            let out = format!("РЕЗУЛЬТАТ:\nРегион: {:?}\np = {:.6} МПа\nT = {:.2} К\nv = {:.6} м3/кг\nrho = {:.2} кг/м3\nh = {:.2} кДж/кг\ns = {:.4} кДж/(кг·К)\nu = {:.2} кДж/кг\ncp = {:.4} кДж/(кг·К)\nw = {:.2} м/с", s.region, s.p.inner(), s.t.inner(), s.v.inner(), s.rho.inner(), s.h.inner(), s.s.inner(), s.u.inner(), s.cp.inner(), s.w.inner());

                            main_ui.single_tab.update_result(&out, false);
                            state.last_single_result = Some(s);
                        }
                        Err(e) => {
                            warn!("Ядро вернуло ошибку: {}", e);
                            main_ui.single_tab.update_result(&format!("Ошибка:\n{}", e), true);
                            state.last_single_result = None;
                        }
                    }
                }

                Message::SaveSinglePoint => {
                    if let Some(st) = &state.last_single_result {
                        if let Some(name) = dialog::input(150, 200, "Имя для точки:", "Точка 1") {
                            if !name.is_empty() {
                                state.datasets.push(SavedData { name: name.clone(), points: vec![st.clone()], visible: true });
                                info!("Точка '{}' сохранена в наборы данных.", name);
                                main_ui.plot_tab.redraw_plot(&state);
                            }
                        }
                    } else {
                        dialog::alert(150, 200, "Сначала выполните успешный расчет!");
                    }
                }

                // --- Табличный расчет ---
                Message::LoadBatchFile => {
                    let mut chooser = dialog::FileChooser::new(".", "*.{csv,txt}", dialog::FileChooserType::Single, "Открыть файл");
                    chooser.show();
                    while chooser.shown() { app::wait(); }
                    if let Some(filename) = chooser.value(1) {
                        if let Ok(content) = fs::read_to_string(&filename) {
                            info!("Загружен файл: {}", filename);
                            main_ui.batch_tab.set_input(&content);
                        }
                    }
                }

                Message::SaveBatchTable => {
                    let current_points = state.datasets[0].points.clone();
                    if current_points.is_empty() {
                        dialog::alert(150, 200, "Таблица пуста!");
                    } else if let Some(name) = dialog::input(150, 200, "Имя набора данных:", "Набор 1") {
                        if !name.is_empty() {
                            state.datasets.push(SavedData { name: name.clone(), points: current_points, visible: true });
                            info!("Набор '{}' сохранен.", name);
                            main_ui.plot_tab.redraw_plot(&state);
                        }
                    }
                }

                Message::BatchDataChanged { mode, content } => {
                    debug!("Пересчет таблицы (режим {})", mode);
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
                                    0 => If97::pt(va.into(), vb.into()),
                                    1 => If97::rhot(va.into(), vb.into()),
                                    2 => If97::ph(va.into(), vb.into()),
                                    3 => If97::ps(va.into(), vb.into()),
                                    4 => If97::px(va.into(), vb.into()),
                                    _ => continue,
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

                                        let fmt = |v: f64, p: usize| if v.is_nan() { "-".into() } else { format!("{:.*}", p, v) };

                                        main_ui.batch_tab.output_table.add(&format!(
                                            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t-\t{}",
                                            fmt(s.p.inner(), 5), fmt(s.t.inner(), 2), rn, xs,
                                            fmt(s.v.inner(), 6), fmt(s.rho.inner(), 2),
                                            fmt(s.h.inner(), 4), fmt(s.s.inner(), 4),
                                            fmt(s.u.inner(), 4), fmt(s.cp.inner(), 4), fmt(s.w.inner(), 2)
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

                // --- Графики и системные команды ---
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
                    let default_path = std::env::var("USERPROFILE")
                        .map(|p| format!("{}\\Desktop\\IF97_plot.png", p))
                        .unwrap_or_else(|_| "IF97_plot.png".to_string());

                    let mut chooser = dialog::FileChooser::new(&default_path, "*.png", dialog::FileChooserType::Create, "Экспорт графика");
                    chooser.show();
                    while chooser.shown() { app::wait(); }
                    if let Some(mut filename) = chooser.value(1) {
                        if !filename.ends_with(".png") { filename.push_str(".png"); }
                        if render_plot_to_file(&state, &filename, 1920, 1080).is_ok() {
                            info!("График успешно экспортирован в {}", filename);
                            dialog::message(150, 200, "График сохранен!");
                        }
                    }
                }

                Message::SaveLogFile => {
                    info!("Запрос на сохранение файла логов.");
                    let mut chooser = dialog::FileChooser::new(".", "Log Files (*.log)", dialog::FileChooserType::Create, "Сохранить отчет об ошибках");
                    chooser.show();
                    while chooser.shown() { app::wait(); }
                    if let Some(mut filename) = chooser.value(1) {
                        if !filename.ends_with(".log") { filename.push_str(".log"); }
                        if let Ok(content) = logs_storage.lock() {
                            if std::fs::write(&filename, content.as_str()).is_ok() {
                                info!("Логи успешно сохранены в {}", filename);
                                dialog::message(150, 200, "Файл логов сохранен!");
                            } else {
                                error!("Ошибка при сохранении логов.");
                            }
                        }
                    }
                }
                // --- Экспорт логов с автоматическим именем ---
                Message::SaveLogFile => {
                    // Генерируем имя вида: if97_debug_2026-03-27_20-45.log
                    let timestamp = Local::now().format("%Y-%m-%d_%H-%M").to_string();
                    let default_name = format!("if97_debug_{}.log", timestamp);

                    info!("Запрос на сохранение логов: {}", default_name);

                    let mut chooser = dialog::FileChooser::new(
                        &default_name,
                        "Log Files (*.log)",
                        dialog::FileChooserType::Create,
                        "Сохранить отчет об ошибках"
                    );
                    chooser.show();
                    while chooser.shown() { app::wait(); }

                    if let Some(filename) = chooser.value(1) {
                        if let Ok(content) = logs_storage.lock() {
                            if std::fs::write(&filename, content.as_str()).is_ok() {
                                dialog::message(150, 200, "Логи успешно сохранены!");
                            }
                        }
                    }
                }

                // --- Экспорт таблицы результатов ---
                Message::ExportBatchData => {
                    let points = &state.datasets[0].points;
                    if points.is_empty() {
                        dialog::alert(150, 200, "Нет данных для экспорта! Сначала введите значения в таблицу.");
                        continue;
                    }

                    let timestamp = Local::now().format("%Y-%m-%d").to_string();
                    let default_name = format!("if97_export_{}.csv", timestamp);

                    let mut chooser = dialog::FileChooser::new(
                        &default_name,
                        "Excel CSV (разделитель ;) (*.csv)\tStandard CSV (разделитель ,) (*.csv)\tText (Tab-separated) (*.txt)",
                        dialog::FileChooserType::Create,
                        "Сохранить результаты расчета"
                    );
                    chooser.show();
                    while chooser.shown() { app::wait(); }

                    if let Some(filename) = chooser.value(1) {
                        // Определяем разделитель по выбранному фильтру
                        let filter = chooser.filter();
                        let delimiter = if filter.as_ref().map_or(false, |f| f.contains("разделитель ;")) {
                            b';'
                        } else if filename.ends_with(".txt") {
                            b'\t'
                        } else {
                            b','
                        };

                        let mut wtr = csv::WriterBuilder::new()
                            .delimiter(delimiter)
                            .from_path(&filename);

                        if let Ok(mut w) = wtr {
                            // Заголовки (используем типичные для отрасли названия)
                            let _ = w.write_record(&["P_MPa", "T_K", "Region", "v_m3_kg", "rho_kg_m3", "h_kJ_kg", "s_kJ_kgK", "u_kJ_kg", "cp_kJ_kgK", "w_m_s"]);
                            for p in points {
                                // Распаковываем значения через .inner()
                                let _ = w.write_record(&[format!("{:.6}", p.p.inner()), format!("{:.2}", p.t.inner()), format!("{:?}", p.region), format!("{:.8}", p.v.inner()), format!("{:.3}", p.rho.inner()), format!("{:.4}", p.h.inner()), format!("{:.5}", p.s.inner()), format!("{:.4}", p.u.inner()), format!("{:.4}", p.cp.inner()), format!("{:.2}", p.w.inner())]);
                            }
                            let _ = w.flush();
                            info!("Таблица успешно экспортирована: {}", filename);
                            dialog::message(150, 200, "Файл успешно сохранен!");
                        } else {
                            error!("Ошибка создания файла: {}", filename);
                            dialog::alert(150, 200, "Не удалось создать файл для записи!");
                        }
                    }
                }
            }
        }
    }
}