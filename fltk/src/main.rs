#![windows_subsystem = "windows"]

//! Нативный desktop-клиент `if97_calculator` на FLTK.

mod plot;
mod state;
mod ui;

use chrono::Local;
use fltk::{app, browser::CheckBrowser, button::Button, dialog, prelude::*, window::Window};
use if97_app_api::{AxisVar, InputMode};
use if97_core::{If97, Region, WaterState, errors::If97Error, saturation};
use std::fs;
use std::io::Write;
use std::sync::{Arc, Mutex};
use tracing::{error, info};
use tracing_subscriber::fmt::writer::MakeWriterExt;

use crate::plot::renderer::{render_plot_to_file, render_plot_to_svg_file};
use crate::state::{AppState, BatchRow, Message, SavedData, SavedKind};
use crate::ui::MainUI;

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

fn parse_value(raw: &str) -> Result<f64, String> {
    raw.trim()
        .replace(',', ".")
        .parse::<f64>()
        .map_err(|_| "Ошибка чтения".to_string())
}

fn normalize_table_input(input: &str) -> String {
    input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn map_error(err: If97Error) -> String {
    match err {
        If97Error::PhaseBoundaryError(msg) => format!("Линия насыщения: {msg}"),
        If97Error::OutOfBounds(msg) => format!("Вне диапазона: {msg}"),
        other => other.to_string(),
    }
}

fn calculate_state(mode: InputMode, raw_a: &str, raw_b: &str) -> Result<WaterState, String> {
    let v1 = parse_value(raw_a)?;
    let v2 = parse_value(raw_b)?;
    let result = match mode {
        InputMode::Pt => If97::pt(v1.into(), v2.into()),
        InputMode::Rhot => If97::rhot(v1.into(), v2.into()),
        InputMode::Ph => If97::ph(v1.into(), v2.into()),
        InputMode::Ps => If97::ps(v1.into(), v2.into()),
        InputMode::Px => If97::px(v1.into(), v2.into()),
    };
    result.map_err(map_error)
}

fn parse_line_pair(line: &str) -> Result<(f64, f64), String> {
    let line = line.trim();
    if line.is_empty() {
        return Err(String::new());
    }

    if line.contains(';') {
        let parts: Vec<&str> = line
            .split(';')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        if parts.len() >= 2 {
            return Ok((parse_value(parts[0])?, parse_value(parts[1])?));
        }
    }

    if line.contains('\t') {
        let parts: Vec<&str> = line
            .split('\t')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .collect();
        if parts.len() >= 2 {
            return Ok((parse_value(parts[0])?, parse_value(parts[1])?));
        }
    }

    let whitespace_parts: Vec<&str> = line.split_whitespace().collect();
    if whitespace_parts.len() >= 2 {
        return Ok((
            parse_value(whitespace_parts[0])?,
            parse_value(whitespace_parts[1])?,
        ));
    }

    let comma_parts: Vec<&str> = line
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    match comma_parts.len() {
        2 => Ok((parse_value(comma_parts[0])?, parse_value(comma_parts[1])?)),
        3 => Ok((
            parse_value(&format!("{},{}", comma_parts[0], comma_parts[1]))?,
            parse_value(comma_parts[2])?,
        )),
        _ => Err("Недостаточно колонок".to_string()),
    }
}

fn calculate_table_rows(mode: InputMode, content: &str) -> Vec<BatchRow> {
    let mut rows = Vec::new();

    for (index, raw_line) in content.lines().enumerate() {
        let line_no = index + 1;
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        let (v1, v2) = match parse_line_pair(line) {
            Ok(values) => values,
            Err(error) => {
                if !error.is_empty() {
                    rows.push(BatchRow {
                        line_no,
                        state: None,
                        error: Some(error),
                    });
                }
                continue;
            }
        };

        match calculate_state(mode, &v1.to_string(), &v2.to_string()) {
            Ok(state) => rows.push(BatchRow {
                line_no,
                state: Some(state),
                error: None,
            }),
            Err(error) => rows.push(BatchRow {
                line_no,
                state: None,
                error: Some(error),
            }),
        }
    }

    rows
}

fn format_value(value: f64, precision: usize, unit: &str) -> String {
    let numeric = if value.is_infinite() {
        if value.is_sign_negative() {
            "-∞".to_string()
        } else {
            "∞".to_string()
        }
    } else if value.is_nan() {
        "NaN".to_string()
    } else {
        format!("{:.*}", precision, value)
    };
    format!("{numeric} {unit}")
}

fn format_single_result(state: &WaterState, precision: usize) -> String {
    [
        "РЕЗУЛЬТАТ:".to_string(),
        format!("Регион: {:?}", state.region),
        format!("p = {}", format_value(state.p.inner(), precision, "МПа")),
        format!("T = {}", format_value(state.t.inner(), precision, "К")),
        format!("v = {}", format_value(state.v.inner(), precision, "м3/кг")),
        format!(
            "rho = {}",
            format_value(state.rho.inner(), precision, "кг/м3")
        ),
        format!("h = {}", format_value(state.h.inner(), precision, "кДж/кг")),
        format!(
            "s = {}",
            format_value(state.s.inner(), precision, "кДж/(кг·К)")
        ),
        format!("u = {}", format_value(state.u.inner(), precision, "кДж/кг")),
        format!(
            "cp = {}",
            format_value(state.cp.inner(), precision, "кДж/(кг·К)")
        ),
        format!("w = {}", format_value(state.w.inner(), precision, "м/с")),
    ]
    .join("\n")
}

fn sync_saved_lists(ui: &mut MainUI, state: &AppState) {
    ui.single_tab.sync_saved_points(&state.saved_point_labels());
    ui.batch_tab.sync_saved_tables(&state.saved_table_labels());
}

fn saved_point_name(state: &AppState, mode: InputMode, raw_a: &str, raw_b: &str) -> Option<String> {
    state
        .datasets
        .iter()
        .find_map(|dataset| match &dataset.kind {
            SavedKind::Point {
                mode: saved_mode,
                v1,
                v2,
            } if *saved_mode == mode && v1 == raw_a && v2 == raw_b => Some(dataset.name.clone()),
            _ => None,
        })
}

fn saved_table_name(state: &AppState, mode: InputMode, content: &str) -> Option<String> {
    let normalized = normalize_table_input(content);
    state
        .datasets
        .iter()
        .find_map(|dataset| match &dataset.kind {
            SavedKind::Table {
                mode: saved_mode,
                input,
            } if *saved_mode == mode && normalize_table_input(input) == normalized => {
                Some(dataset.name.clone())
            }
            _ => None,
        })
}

fn refresh_single_result(
    ui: &mut MainUI,
    state: &mut AppState,
    mode: InputMode,
    raw_a: String,
    raw_b: String,
) {
    if raw_a.trim().is_empty() || raw_b.trim().is_empty() {
        ui.single_tab.update_result("Ожидание данных...", false);
        state.last_single_result = None;
        state.last_single_request = None;
        return;
    }

    match calculate_state(mode, &raw_a, &raw_b) {
        Ok(result) => {
            ui.single_tab.update_result(
                &format_single_result(&result, state.single_precision),
                false,
            );
            state.last_single_result = Some(result);
            state.last_single_request = Some((mode, raw_a.clone(), raw_b.clone()));
            if let Some(name) = saved_point_name(state, mode, &raw_a, &raw_b) {
                ui.single_tab.set_save_name(&name);
            }
        }
        Err(error) => {
            ui.single_tab
                .update_result(&format!("Ошибка расчета:\n{error}"), true);
            state.last_single_result = None;
            state.last_single_request = Some((mode, raw_a, raw_b));
        }
    }
}

fn refresh_batch_result(ui: &mut MainUI, state: &mut AppState, mode: InputMode, content: String) {
    state.current_table_rows = calculate_table_rows(mode, &content);
    state.current_table_dataset_mut().points = state
        .current_table_rows
        .iter()
        .filter_map(|row| row.state.clone())
        .collect();

    ui.batch_tab.update_table(
        &state.current_table_rows,
        state.table_precision,
        state.table_scientific,
    );

    if let Some(name) = saved_table_name(state, mode, &content) {
        ui.batch_tab.set_save_name(&name);
    }
}

fn generate_table_block(
    v1_from: &str,
    v1_to: &str,
    v1_step: &str,
    v2_from: &str,
    v2_to: &str,
    v2_step: &str,
) -> Result<String, String> {
    let parse_range =
        |from: &str, to: &str, step: &str| -> Result<(f64, f64, f64, usize), String> {
            let from = parse_value(from)?;
            let to = parse_value(to)?;
            let step = parse_value(step)?;
            if step == 0.0 {
                return Err("Шаг генератора не может быть нулевым".to_string());
            }
            if (to > from && step < 0.0) || (to < from && step > 0.0) {
                return Err("Шаг генератора не соответствует направлению диапазона".to_string());
            }
            let steps = (((to - from) / step) + 1e-9).abs().floor() as usize;
            Ok((from, to, step, steps))
        };

    let (from1, _, step1, steps1) = parse_range(v1_from, v1_to, v1_step)?;
    let (from2, _, step2, steps2) = parse_range(v2_from, v2_to, v2_step)?;

    let total_rows = (steps1 + 1).saturating_mul(steps2 + 1);
    if total_rows > 20_000 {
        return Err("Превышен лимит генератора: максимум 20000 строк".to_string());
    }

    let mut generated = String::new();
    for index1 in 0..=steps1 {
        let value1 = from1 + (index1 as f64) * step1;
        let value1 = (value1 * 1_000_000_000.0).round() / 1_000_000_000.0;
        for index2 in 0..=steps2 {
            let value2 = from2 + (index2 as f64) * step2;
            let value2 = (value2 * 1_000_000_000.0).round() / 1_000_000_000.0;
            generated.push_str(&format!("{value1};{value2}\n"));
        }
    }

    Ok(generated)
}

fn write_batch_export(
    state: &AppState,
    filename: &str,
    delimiter: u8,
) -> Result<(), Box<dyn std::error::Error>> {
    fn format_export_value(value: f64, precision: usize, scientific: bool) -> String {
        if value.is_nan() {
            "NaN".to_string()
        } else if value.is_infinite() {
            if value.is_sign_negative() {
                "-inf".to_string()
            } else {
                "inf".to_string()
            }
        } else if scientific {
            format!("{:.*e}", precision, value)
        } else {
            format!("{:.*}", precision, value)
        }
    }

    fn sanitize_cp(value: f64) -> f64 {
        if value.is_finite() {
            return value;
        }

        // Для экспорта не пишем NaN/inf, а отдаём большое конечное число.
        const CAP: f64 = 1e12;
        if value.is_sign_negative() { -CAP } else { CAP }
    }

    fn quality_x(point: &WaterState) -> f64 {
        if point.region != Region::Region4 {
            return f64::NAN;
        }

        let p = point.p;
        let v = point.v.inner();
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

    fn cp_value(point: &WaterState, x_hint: f64) -> f64 {
        let raw = point.cp.inner();
        if raw.is_finite() {
            return raw;
        }

        if point.region != Region::Region4 {
            return sanitize_cp(raw);
        }

        let x = if x_hint.is_finite() {
            x_hint
        } else {
            quality_x(point)
        };
        if !x.is_finite() {
            return sanitize_cp(raw);
        }

        let p = point.p;
        let Ok(liq) = saturation::saturated_liquid(p) else {
            return sanitize_cp(raw);
        };
        let Ok(vap) = saturation::saturated_vapor(p) else {
            return sanitize_cp(raw);
        };

        let cp_liq = liq.cp.inner();
        let cp_vap = vap.cp.inner();
        let cp = cp_liq + x * (cp_vap - cp_liq);
        if cp.is_finite() { cp } else { sanitize_cp(raw) }
    }

    let mut writer = csv::WriterBuilder::new()
        .delimiter(delimiter)
        .from_path(filename)?;

    writer.write_record([
        "строка",
        "p_MPa",
        "T_K",
        "v_m3_kg",
        "rho_kg_m3",
        "h_kJ_kg",
        "s_kJ_kgK",
        "u_kJ_kg",
        "cp_kJ_kgK",
        "w_m_s",
        "x_vapor_fraction",
        "region",
        "error",
    ])?;

    for row in &state.current_table_rows {
        if let Some(point) = &row.state {
            let x = quality_x(point);
            let cp = cp_value(point, x);
            writer.write_record([
                row.line_no.to_string(),
                format_export_value(
                    point.p.inner(),
                    state.table_precision,
                    state.table_scientific,
                ),
                format_export_value(
                    point.t.inner(),
                    state.table_precision,
                    state.table_scientific,
                ),
                format_export_value(
                    point.v.inner(),
                    state.table_precision,
                    state.table_scientific,
                ),
                format_export_value(
                    point.rho.inner(),
                    state.table_precision,
                    state.table_scientific,
                ),
                format_export_value(
                    point.h.inner(),
                    state.table_precision,
                    state.table_scientific,
                ),
                format_export_value(
                    point.s.inner(),
                    state.table_precision,
                    state.table_scientific,
                ),
                format_export_value(
                    point.u.inner(),
                    state.table_precision,
                    state.table_scientific,
                ),
                format_export_value(cp, state.table_precision, state.table_scientific),
                format_export_value(
                    point.w.inner(),
                    state.table_precision,
                    state.table_scientific,
                ),
                format_export_value(x, state.table_precision, state.table_scientific),
                format!("{:?}", point.region),
                String::new(),
            ])?;
        } else {
            writer.write_record([
                row.line_no.to_string(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                String::new(),
                row.error.clone().unwrap_or_default(),
            ])?;
        }
    }

    writer.flush()?;
    Ok(())
}

fn main() {
    let logs_storage = Arc::new(Mutex::new(String::new()));
    let log_adapter = LogBuffer(logs_storage.clone());

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(std::io::stderr.and(move || log_adapter.clone()))
        .init();

    info!("Приложение FLTK запущено");

    let fltk_app = app::App::default();
    let (sender, receiver) = app::channel::<Message>();

    let mut state = AppState::new();
    let mut main_ui = MainUI::new(sender.clone());
    main_ui.single_tab.set_mode(InputMode::Pt);
    main_ui.single_tab.set_precision(state.single_precision);
    main_ui.batch_tab.set_mode(InputMode::Pt);
    main_ui.batch_tab.set_precision(state.table_precision);
    main_ui.batch_tab.set_scientific(state.table_scientific);
    main_ui.plot_tab.redraw_plot(&state);
    sync_saved_lists(&mut main_ui, &state);

    main_ui.window.show();

    while fltk_app.wait() {
        if let Some(msg) = receiver.recv() {
            match msg {
                Message::CalculateSingle { mode, raw_a, raw_b } => {
                    refresh_single_result(&mut main_ui, &mut state, mode, raw_a, raw_b);
                }
                Message::SetSinglePrecision(precision) => {
                    state.single_precision = precision.min(10);
                    if let Some(result) = &state.last_single_result {
                        main_ui.single_tab.update_result(
                            &format_single_result(result, state.single_precision),
                            false,
                        );
                    }
                }
                Message::SaveSinglePoint {
                    name,
                    mode,
                    raw_a,
                    raw_b,
                } => {
                    let Some(result) = state.last_single_result.clone() else {
                        dialog::alert(150, 200, "Сначала выполните успешный расчет.");
                        continue;
                    };

                    let name = if name.trim().is_empty() {
                        format!("Точка {}", state.saved_point_labels().len() + 1)
                    } else {
                        name
                    };

                    if let Some(existing) = state.datasets.iter_mut().find(|dataset| {
                        matches!(
                            &dataset.kind,
                            SavedKind::Point { mode: saved_mode, v1, v2 }
                                if *saved_mode == mode && v1 == &raw_a && v2 == &raw_b
                        )
                    }) {
                        existing.name = name.clone();
                        existing.points = vec![result];
                    } else {
                        state.datasets.push(SavedData {
                            name: name.clone(),
                            points: vec![result],
                            visible: true,
                            kind: SavedKind::Point {
                                mode,
                                v1: raw_a.clone(),
                                v2: raw_b.clone(),
                            },
                        });
                    }

                    main_ui.single_tab.set_save_name(&name);
                    sync_saved_lists(&mut main_ui, &state);
                    main_ui.plot_tab.redraw_plot(&state);
                }
                Message::LoadSavedPoint(index) => {
                    if let Some(dataset) = state.saved_point_at(index).cloned() {
                        if let SavedKind::Point { mode, v1, v2 } = dataset.kind {
                            main_ui.single_tab.set_request(mode, &v1, &v2);
                            main_ui.single_tab.set_save_name(&dataset.name);
                            refresh_single_result(&mut main_ui, &mut state, mode, v1, v2);
                        }
                    }
                }
                Message::DeleteSavedPoint(index) => {
                    state.remove_saved_point(index);
                    sync_saved_lists(&mut main_ui, &state);
                    main_ui.plot_tab.redraw_plot(&state);
                }
                Message::LoadBatchFile => {
                    let mut chooser = dialog::FileChooser::new(
                        ".",
                        "*.{csv,txt,tsv,dat}",
                        dialog::FileChooserType::Single,
                        "Открыть файл",
                    );
                    chooser.show();
                    while chooser.shown() {
                        app::wait();
                    }
                    if let Some(filename) = chooser.value(1) {
                        match fs::read_to_string(&filename) {
                            Ok(content) => {
                                main_ui.batch_tab.set_input(&content);
                                let mode = match main_ui.batch_tab.choice_mode.value() {
                                    0 => InputMode::Pt,
                                    1 => InputMode::Rhot,
                                    2 => InputMode::Ph,
                                    3 => InputMode::Ps,
                                    _ => InputMode::Px,
                                };
                                refresh_batch_result(&mut main_ui, &mut state, mode, content);
                                main_ui.plot_tab.redraw_plot(&state);
                            }
                            Err(err) => {
                                dialog::alert(150, 200, &format!("Ошибка чтения файла: {err}"))
                            }
                        }
                    }
                }
                Message::SetTablePrecision(precision) => {
                    state.table_precision = precision.min(10);
                    main_ui.batch_tab.update_table(
                        &state.current_table_rows,
                        state.table_precision,
                        state.table_scientific,
                    );
                }
                Message::SetTableScientific(enabled) => {
                    state.table_scientific = enabled;
                    main_ui.batch_tab.set_scientific(enabled);
                    main_ui.batch_tab.update_table(
                        &state.current_table_rows,
                        state.table_precision,
                        state.table_scientific,
                    );
                }
                Message::SaveBatchTable {
                    name,
                    mode,
                    content,
                } => {
                    let points = state
                        .current_table_rows
                        .iter()
                        .filter_map(|row| row.state.clone())
                        .collect::<Vec<_>>();
                    if points.is_empty() {
                        dialog::alert(150, 200, "Нет корректных строк для сохранения.");
                        continue;
                    }

                    let name = if name.trim().is_empty() {
                        format!("Таблица {}", state.saved_table_labels().len() + 1)
                    } else {
                        name
                    };

                    if let Some(existing) = state.datasets.iter_mut().find(|dataset| {
                        matches!(
                            &dataset.kind,
                            SavedKind::Table { mode: saved_mode, input }
                                if *saved_mode == mode
                                    && normalize_table_input(input) == normalize_table_input(&content)
                        )
                    }) {
                        existing.name = name.clone();
                        existing.points = points;
                    } else {
                        state.datasets.push(SavedData {
                            name: name.clone(),
                            points,
                            visible: true,
                            kind: SavedKind::Table {
                                mode,
                                input: content.clone(),
                            },
                        });
                    }

                    main_ui.batch_tab.set_save_name(&name);
                    sync_saved_lists(&mut main_ui, &state);
                    main_ui.plot_tab.redraw_plot(&state);
                }
                Message::LoadSavedTable(index) => {
                    if let Some(dataset) = state.saved_table_at(index).cloned() {
                        if let SavedKind::Table { mode, input } = dataset.kind {
                            main_ui.batch_tab.set_mode(mode);
                            main_ui.batch_tab.set_input(&input);
                            main_ui.batch_tab.set_save_name(&dataset.name);
                            refresh_batch_result(&mut main_ui, &mut state, mode, input);
                            main_ui.plot_tab.redraw_plot(&state);
                        }
                    }
                }
                Message::DeleteSavedTable(index) => {
                    state.remove_saved_table(index);
                    sync_saved_lists(&mut main_ui, &state);
                    main_ui.plot_tab.redraw_plot(&state);
                }
                Message::BatchDataChanged { mode, content } => {
                    refresh_batch_result(&mut main_ui, &mut state, mode, content);
                    main_ui.plot_tab.redraw_plot(&state);
                }
                Message::GenerateBatchData {
                    v1_from,
                    v1_to,
                    v1_step,
                    v2_from,
                    v2_to,
                    v2_step,
                } => match generate_table_block(
                    &v1_from, &v1_to, &v1_step, &v2_from, &v2_to, &v2_step,
                ) {
                    Ok(generated) => {
                        let mut content = main_ui.batch_tab.input_area.value();
                        if !content.is_empty() && !content.ends_with('\n') {
                            content.push('\n');
                        }
                        content.push_str(&generated);
                        main_ui.batch_tab.set_input(&content);
                        let mode = match main_ui.batch_tab.choice_mode.value() {
                            0 => InputMode::Pt,
                            1 => InputMode::Rhot,
                            2 => InputMode::Ph,
                            3 => InputMode::Ps,
                            _ => InputMode::Px,
                        };
                        refresh_batch_result(&mut main_ui, &mut state, mode, content);
                        main_ui.plot_tab.redraw_plot(&state);
                    }
                    Err(error) => dialog::alert(150, 200, &error),
                },
                Message::SetPlotX(var) => {
                    state.plot_x = var;
                    if state.autoscale {
                        let (min, max) = default_range(var);
                        state.custom_limits =
                            (min, max, state.custom_limits.2, state.custom_limits.3);
                    }
                    main_ui.plot_tab.redraw_plot(&state);
                }
                Message::SetPlotY(var) => {
                    state.plot_y = var;
                    if state.autoscale {
                        let (min, max) = default_range(var);
                        state.custom_limits =
                            (state.custom_limits.0, state.custom_limits.1, min, max);
                    }
                    main_ui.plot_tab.redraw_plot(&state);
                }
                Message::ToggleDome(show) => {
                    state.show_dome = show;
                    main_ui.plot_tab.redraw_plot(&state);
                }
                Message::TogglePlotXLog(enabled) => {
                    state.plot_x_log = enabled;
                    main_ui.plot_tab.redraw_plot(&state);
                }
                Message::TogglePlotYLog(enabled) => {
                    state.plot_y_log = enabled;
                    main_ui.plot_tab.redraw_plot(&state);
                }
                Message::SetAutoscale(auto) => {
                    state.autoscale = auto;
                    main_ui.plot_tab.redraw_plot(&state);
                }
                Message::ApplyPlotLimits(x_min, x_max, y_min, y_max) => {
                    state.custom_limits = (x_min, x_max, y_min, y_max);
                    main_ui.plot_tab.redraw_plot(&state);
                }
                Message::SelectData => {
                    let mut win = Window::default()
                        .with_size(320, 420)
                        .with_label("Выбор данных");
                    let mut cb = CheckBrowser::default().with_size(300, 340).with_pos(10, 10);
                    let mut btn_apply = Button::default()
                        .with_size(100, 40)
                        .with_label("Применить")
                        .with_pos(110, 360);
                    for dataset in &state.datasets {
                        cb.add(&dataset.name, dataset.visible);
                    }
                    win.make_modal(true);
                    win.show();
                    let s_cloned = sender.clone();
                    btn_apply.set_callback(move |b| {
                        let mut vis = Vec::new();
                        for i in 1..=cb.nitems() {
                            vis.push(cb.checked(i as i32));
                        }
                        s_cloned.send(Message::UpdateDataVisibility(vis));
                        b.window().unwrap().hide();
                    });
                }
                Message::UpdateDataVisibility(vis) => {
                    for (index, visible) in vis.into_iter().enumerate() {
                        if let Some(dataset) = state.datasets.get_mut(index) {
                            dataset.visible = visible;
                        }
                    }
                    main_ui.plot_tab.redraw_plot(&state);
                }
                Message::ExportPlot => {
                    let default_path = std::env::var("HOME")
                        .map(|p| format!("{}/Desktop/if97_plot.png", p))
                        .unwrap_or_else(|_| "if97_plot.png".to_string());

                    let mut chooser = dialog::FileChooser::new(
                        &default_path,
                        "*.png",
                        dialog::FileChooserType::Create,
                        "Экспорт графика",
                    );
                    chooser.show();
                    while chooser.shown() {
                        app::wait();
                    }
                    if let Some(mut filename) = chooser.value(1) {
                        if !filename.to_lowercase().ends_with(".png") {
                            filename.push_str(".png");
                        }
                        match render_plot_to_file(&state, &filename, 1920, 1080) {
                            Ok(_) => dialog::message(150, 200, "График сохранен."),
                            Err(err) => dialog::alert(150, 200, &format!("Ошибка экспорта: {err}")),
                        }
                    }
                }
                Message::ExportPlotSvg => {
                    let default_path = std::env::var("HOME")
                        .map(|p| format!("{}/Desktop/if97_plot.svg", p))
                        .unwrap_or_else(|_| "if97_plot.svg".to_string());

                    let mut chooser = dialog::FileChooser::new(
                        &default_path,
                        "*.svg",
                        dialog::FileChooserType::Create,
                        "Экспорт графика",
                    );
                    chooser.show();
                    while chooser.shown() {
                        app::wait();
                    }
                    if let Some(mut filename) = chooser.value(1) {
                        if !filename.to_lowercase().ends_with(".svg") {
                            filename.push_str(".svg");
                        }
                        match render_plot_to_svg_file(&state, &filename, 1920, 1080) {
                            Ok(_) => dialog::message(150, 200, "График сохранен."),
                            Err(err) => dialog::alert(150, 200, &format!("Ошибка экспорта: {err}")),
                        }
                    }
                }
                Message::SaveLogFile => {
                    let timestamp = Local::now().format("%Y-%m-%d_%H-%M").to_string();
                    let default_name = format!("if97_debug_{timestamp}.log");

                    let mut chooser = dialog::FileChooser::new(
                        &default_name,
                        "Log Files (*.log)",
                        dialog::FileChooserType::Create,
                        "Сохранить журнал",
                    );
                    chooser.show();
                    while chooser.shown() {
                        app::wait();
                    }

                    if let Some(mut filename) = chooser.value(1) {
                        if !filename.ends_with(".log") {
                            filename.push_str(".log");
                        }
                        if let Ok(content) = logs_storage.lock() {
                            match std::fs::write(&filename, content.as_str()) {
                                Ok(_) => dialog::message(150, 200, "Логи сохранены."),
                                Err(err) => {
                                    dialog::alert(150, 200, &format!("Ошибка записи: {err}"))
                                }
                            }
                        }
                    }
                }
                Message::ExportBatchData => {
                    if state.current_table_rows.is_empty() {
                        dialog::alert(150, 200, "Нет данных для экспорта.");
                        continue;
                    }

                    let timestamp = Local::now().format("%Y-%m-%d").to_string();
                    let default_name = format!("if97_export_{timestamp}.csv");

                    let mut chooser = dialog::FileChooser::new(
                        &default_name,
                        "Excel CSV (разделитель ;) (*.csv)\tStandard CSV (разделитель ,) (*.csv)\tText (Tab-separated) (*.txt)",
                        dialog::FileChooserType::Create,
                        "Сохранить результаты расчета",
                    );
                    chooser.show();
                    while chooser.shown() {
                        app::wait();
                    }

                    if let Some(filename) = chooser.value(1) {
                        let filter = chooser.filter();
                        let delimiter = if filter
                            .as_ref()
                            .map_or(false, |f| f.contains("разделитель ;"))
                        {
                            b';'
                        } else if filename.ends_with(".txt") {
                            b'\t'
                        } else {
                            b','
                        };

                        match write_batch_export(&state, &filename, delimiter) {
                            Ok(_) => dialog::message(150, 200, "Файл успешно сохранен."),
                            Err(err) => {
                                error!("Ошибка экспорта таблицы: {err}");
                                dialog::alert(150, 200, &format!("Ошибка экспорта: {err}"));
                            }
                        }
                    }
                }
            }
        }
    }
}
