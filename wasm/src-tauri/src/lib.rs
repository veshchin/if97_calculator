//! Tauri-backend для desktop-сборки `if97_calculator`.
//!
//! Крейт предоставляет набор `#[tauri::command]` для UI-frontend (Yew) и запускает
//! приложение через `tauri::Builder`.

use base64::{engine::general_purpose::STANDARD, Engine as _};
use if97_app_api::{
    DiagramKind, DomeRequest, InputMode, LogEntryDto, PlotPoint, SingleCalcRequest, StateDto,
    TableCalcRequest, TableRowResult,
};
use if97_core::{errors::If97Error, saturation, If97, WaterState};
use once_cell::sync::Lazy;
use std::collections::VecDeque;
use std::fs;
use std::sync::Mutex;
use tauri::async_runtime;
use tauri_plugin_dialog::DialogExt;
use tracing_subscriber::Registry;
use tracing_subscriber::layer::SubscriberExt;

const MAX_LOG_ENTRIES: usize = 4000;

static LOG_BUFFER: Lazy<Mutex<VecDeque<LogEntryDto>>> =
    Lazy::new(|| Mutex::new(VecDeque::with_capacity(MAX_LOG_ENTRIES)));

struct BackendLogLayer;

struct StringVisitor {
    message: String,
}

impl tracing::field::Visit for StringVisitor {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        }
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" && self.message.is_empty() {
            self.message = format!("{value:?}");
        }
    }
}

impl<S> tracing_subscriber::Layer<S> for BackendLogLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let metadata = event.metadata();
        let target = metadata.target();

        if target.starts_with("tao")
            || target.starts_with("wry")
            || target.starts_with("mio")
            || target.starts_with("hyper")
        {
            return;
        }

        let mut visitor = StringVisitor {
            message: String::new(),
        };
        event.record(&mut visitor);

        let message = if visitor.message.trim().is_empty() {
            metadata.name().to_string()
        } else {
            visitor.message
        };

        let source = if target.starts_with("if97_core") {
            "core"
        } else {
            "backend"
        };

        if let Ok(mut buffer) = LOG_BUFFER.lock() {
            if buffer.len() == MAX_LOG_ENTRIES {
                buffer.pop_front();
            }
            buffer.push_back(LogEntryDto {
                source: source.to_string(),
                level: metadata.level().to_string().to_lowercase(),
                message: format!("[{source}][{}] {message}", metadata.level()),
            });
        }
    }
}

fn init_logging() {
    let subscriber = Registry::default()
        .with(tracing_subscriber::filter::LevelFilter::TRACE)
        .with(BackendLogLayer);
    let _ = tracing::subscriber::set_global_default(subscriber);
    tracing::info!("инициализация журналирования завершена");
}

fn snapshot_logs() -> Vec<LogEntryDto> {
    LOG_BUFFER
        .lock()
        .map(|buffer| buffer.iter().cloned().collect())
        .unwrap_or_default()
}

fn clear_logs_buffer() {
    if let Ok(mut buffer) = LOG_BUFFER.lock() {
        buffer.clear();
    }
}

fn map_error(err: If97Error) -> String {
    match err {
        If97Error::PhaseBoundaryError(msg) => format!("Линия насыщения: {msg}"),
        If97Error::OutOfBounds(msg) => format!("Вне диапазона: {msg}"),
        other => other.to_string(),
    }
}

fn to_state_dto(state: WaterState) -> StateDto {
    StateDto {
        p: state.p.inner(),
        t: state.t.inner(),
        v: state.v.inner(),
        rho: state.rho.inner(),
        h: state.h.inner(),
        s: state.s.inner(),
        u: state.u.inner(),
        cp: state.cp.inner(),
        w: state.w.inner(),
        region: format!("{:?}", state.region),
    }
}

fn calculate_state(request: SingleCalcRequest) -> Result<StateDto, String> {
    let result = match request.mode {
        InputMode::Pt => If97::pt(request.v1.into(), request.v2.into()),
        InputMode::Ph => If97::ph(request.v1.into(), request.v2.into()),
        InputMode::Ps => If97::ps(request.v1.into(), request.v2.into()),
        InputMode::Px => If97::px(request.v1.into(), request.v2.into()),
        InputMode::Rhot => If97::rhot(request.v1.into(), request.v2.into()),
    };

    result.map(to_state_dto).map_err(map_error)
}

fn parse_value(raw: &str) -> Result<f64, String> {
    raw.trim()
        .replace(',', ".")
        .parse::<f64>()
        .map_err(|_| "Ошибка чтения".to_string())
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

fn calculate_table_rows(request: TableCalcRequest) -> Vec<TableRowResult> {
    let mut rows = Vec::new();

    for (index, raw_line) in request.input.lines().enumerate() {
        let line_no = index + 1;
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        let (v1, v2) = match parse_line_pair(line) {
            Ok(values) => values,
            Err(error) => {
                if !error.is_empty() {
                    rows.push(TableRowResult {
                        line_no,
                        state: None,
                        error: Some(error),
                    });
                }
                continue;
            }
        };

        let row = match calculate_state(SingleCalcRequest {
            mode: request.mode,
            v1,
            v2,
        }) {
            Ok(state) => TableRowResult {
                line_no,
                state: Some(state),
                error: None,
            },
            Err(error) => TableRowResult {
                line_no,
                state: None,
                error: Some(error),
            },
        };

        rows.push(row);
    }

    rows
}

fn project_state(chart_type: DiagramKind, swap_axes: bool, state: WaterState) -> PlotPoint {
    let (mut x, mut y) = match chart_type {
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

    PlotPoint { x, y }
}

#[derive(Clone, Copy)]
enum AxisVar {
    P,
    T,
    H,
    S,
    V,
}

fn axis_vars(chart_type: DiagramKind, swap_axes: bool) -> (AxisVar, AxisVar) {
    let (mut x, mut y) = match chart_type {
        DiagramKind::Ts => (AxisVar::S, AxisVar::T),
        DiagramKind::Hs => (AxisVar::S, AxisVar::H),
        DiagramKind::Ph => (AxisVar::H, AxisVar::P),
        DiagramKind::Tv => (AxisVar::V, AxisVar::T),
        DiagramKind::Pv => (AxisVar::V, AxisVar::P),
        DiagramKind::Pt => (AxisVar::T, AxisVar::P),
        DiagramKind::Ps => (AxisVar::S, AxisVar::P),
        DiagramKind::Th => (AxisVar::H, AxisVar::T),
    };

    if swap_axes {
        std::mem::swap(&mut x, &mut y);
    }

    (x, y)
}

fn metric_value(var: AxisVar, value: f64) -> f64 {
    // Для v-диаграмм размах по оси v большой, из-за чего адаптивное уточнение
    // недодаёт точек в области малого v. В метрике используем ln(v).
    match var {
        AxisVar::V => value.max(1e-12).ln(),
        _ => value,
    }
}

const SAT_P_MIN: f64 = 0.000611657_f64;
const SAT_P_MAX: f64 = 22.064_f64;
const SAT_MAX_DEPTH: usize = 12;
const SAT_REL_TOL_DEFAULT: f64 = 0.00035;
const SAT_REL_TOL_V: f64 = 0.00015;
const SAT_PRE_SAMPLES: usize = 32;

fn saturation_pressure(t: f64) -> f64 {
    (SAT_P_MIN.ln() + t * (SAT_P_MAX.ln() - SAT_P_MIN.ln())).exp()
}

fn eval_saturation_point(
    chart_type: DiagramKind,
    swap_axes: bool,
    quality: f64,
    t: f64,
) -> Option<PlotPoint> {
    let pressure = saturation_pressure(t);
    let state = if quality <= 0.0 {
        saturation::saturated_liquid(pressure.into()).ok()?
    } else {
        saturation::saturated_vapor(pressure.into()).ok()?
    };
    Some(project_state(chart_type, swap_axes, state))
}

fn estimate_dome_spans(chart_type: DiagramKind, swap_axes: bool) -> (f64, f64, f64, f64) {
    let mut min_lin_x = f64::INFINITY;
    let mut max_lin_x = f64::NEG_INFINITY;
    let mut min_lin_y = f64::INFINITY;
    let mut max_lin_y = f64::NEG_INFINITY;

    let mut min_met_x = f64::INFINITY;
    let mut max_met_x = f64::NEG_INFINITY;
    let mut min_met_y = f64::INFINITY;
    let mut max_met_y = f64::NEG_INFINITY;

    let (x_var, y_var) = axis_vars(chart_type, swap_axes);
    let qualities = [0.0, 1.0];
    for &quality in &qualities {
        for i in 0..SAT_PRE_SAMPLES {
            let t = if SAT_PRE_SAMPLES <= 1 {
                0.0
            } else {
                i as f64 / (SAT_PRE_SAMPLES - 1) as f64
            };
            if let Some(p) = eval_saturation_point(chart_type, swap_axes, quality, t) {
                if p.x.is_finite() && p.y.is_finite() {
                    min_lin_x = min_lin_x.min(p.x);
                    max_lin_x = max_lin_x.max(p.x);
                    min_lin_y = min_lin_y.min(p.y);
                    max_lin_y = max_lin_y.max(p.y);

                    let mx = metric_value(x_var, p.x);
                    let my = metric_value(y_var, p.y);
                    min_met_x = min_met_x.min(mx);
                    max_met_x = max_met_x.max(mx);
                    min_met_y = min_met_y.min(my);
                    max_met_y = max_met_y.max(my);
                }
            }
        }
    }

    if !min_lin_x.is_finite() || !min_lin_y.is_finite() {
        return (1.0, 1.0, 1.0, 1.0);
    }

    let lin_span_x = (max_lin_x - min_lin_x).abs().max(1e-9);
    let lin_span_y = (max_lin_y - min_lin_y).abs().max(1e-9);

    let met_span_x = if min_met_x.is_finite() {
        (max_met_x - min_met_x).abs().max(1e-9)
    } else {
        lin_span_x
    };
    let met_span_y = if min_met_y.is_finite() {
        (max_met_y - min_met_y).abs().max(1e-9)
    } else {
        lin_span_y
    };

    (lin_span_x, lin_span_y, met_span_x, met_span_y)
}

fn refine_saturation_segment(
    chart_type: DiagramKind,
    swap_axes: bool,
    quality: f64,
    x_var: AxisVar,
    y_var: AxisVar,
    lin_span_x: f64,
    lin_span_y: f64,
    met_span_x: f64,
    met_span_y: f64,
    t0: f64,
    p0: PlotPoint,
    t1: f64,
    p1: PlotPoint,
    depth: usize,
    out: &mut Vec<PlotPoint>,
) {
    if depth >= SAT_MAX_DEPTH {
        out.push(p0);
        return;
    }

    let mid_t = 0.5 * (t0 + t1);
    let Some(pm) = eval_saturation_point(chart_type, swap_axes, quality, mid_t) else {
        out.push(p0);
        return;
    };

    let lin_dx = (p1.x - p0.x) / lin_span_x;
    let lin_dy = (p1.y - p0.y) / lin_span_y;
    let lin_seg_len = (lin_dx * lin_dx + lin_dy * lin_dy).sqrt();
    let lin_mx = (pm.x - p0.x) / lin_span_x;
    let lin_my = (pm.y - p0.y) / lin_span_y;
    let lin_dev = if lin_seg_len <= f64::EPSILON {
        0.0
    } else {
        (lin_dx * lin_my - lin_dy * lin_mx).abs() / lin_seg_len
    };

    let m0x = metric_value(x_var, p0.x);
    let m0y = metric_value(y_var, p0.y);
    let m1x = metric_value(x_var, p1.x);
    let m1y = metric_value(y_var, p1.y);
    let mmx = metric_value(x_var, pm.x);
    let mmy = metric_value(y_var, pm.y);

    let met_dx = (m1x - m0x) / met_span_x;
    let met_dy = (m1y - m0y) / met_span_y;
    let met_seg_len = (met_dx * met_dx + met_dy * met_dy).sqrt();
    let met_mx = (mmx - m0x) / met_span_x;
    let met_my = (mmy - m0y) / met_span_y;
    let met_dev = if met_seg_len <= f64::EPSILON {
        0.0
    } else {
        (met_dx * met_my - met_dy * met_mx).abs() / met_seg_len
    };

    let rel_tol = if matches!(x_var, AxisVar::V) || matches!(y_var, AxisVar::V) {
        SAT_REL_TOL_V
    } else {
        SAT_REL_TOL_DEFAULT
    };
    let need_refine = (lin_seg_len > 0.0 && lin_dev > lin_seg_len * rel_tol)
        || (met_seg_len > 0.0 && met_dev > met_seg_len * rel_tol);

    if need_refine {
        refine_saturation_segment(
            chart_type,
            swap_axes,
            quality,
            x_var,
            y_var,
            lin_span_x,
            lin_span_y,
            met_span_x,
            met_span_y,
            t0,
            p0,
            mid_t,
            pm,
            depth + 1,
            out,
        );
        refine_saturation_segment(
            chart_type,
            swap_axes,
            quality,
            x_var,
            y_var,
            lin_span_x,
            lin_span_y,
            met_span_x,
            met_span_y,
            mid_t,
            pm,
            t1,
            p1,
            depth + 1,
            out,
        );
    } else {
        out.push(p0);
    }
}

fn build_saturation_side(chart_type: DiagramKind, swap_axes: bool, quality: f64) -> Vec<PlotPoint> {
    let Some(p0) = eval_saturation_point(chart_type, swap_axes, quality, 0.0) else {
        return Vec::new();
    };
    let Some(p1) = eval_saturation_point(chart_type, swap_axes, quality, 1.0) else {
        return Vec::new();
    };

    let (x_var, y_var) = axis_vars(chart_type, swap_axes);
    let (lin_span_x, lin_span_y, met_span_x, met_span_y) =
        estimate_dome_spans(chart_type, swap_axes);
    let mut points = Vec::new();
    refine_saturation_segment(
        chart_type,
        swap_axes,
        quality,
        x_var,
        y_var,
        lin_span_x,
        lin_span_y,
        met_span_x,
        met_span_y,
        0.0,
        p0,
        1.0,
        p1,
        0,
        &mut points,
    );
    points.push(p1);
    points
}

fn calculate_dome_points(request: DomeRequest) -> Vec<PlotPoint> {
    let liquid = build_saturation_side(request.chart_type, request.swap_axes, 0.0);
    if liquid.is_empty() {
        return Vec::new();
    }

    // В p-T диаграмме линия насыщения совпадает для x=0 и x=1, поэтому не дублируем путь.
    if request.chart_type == DiagramKind::Pt {
        return liquid;
    }

    let mut vapor = build_saturation_side(request.chart_type, request.swap_axes, 1.0);
    vapor.reverse();

    let mut dome = liquid;
    dome.extend(vapor);
    dome
}

#[tauri::command]
async fn read_logs() -> Result<Vec<LogEntryDto>, String> {
    Ok(snapshot_logs())
}

#[tauri::command]
async fn clear_logs() -> Result<(), String> {
    clear_logs_buffer();
    Ok(())
}

#[tauri::command]
async fn calculate_single(request: SingleCalcRequest) -> Result<StateDto, String> {
    async_runtime::spawn_blocking(move || calculate_state(request))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn calculate_table(request: TableCalcRequest) -> Result<Vec<TableRowResult>, String> {
    async_runtime::spawn_blocking(move || calculate_table_rows(request))
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn calculate_dome(request: DomeRequest) -> Result<Vec<PlotPoint>, String> {
    async_runtime::spawn_blocking(move || calculate_dome_points(request))
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn load_file_dialog(app: tauri::AppHandle) -> Result<String, String> {
    let file_path = app
        .dialog()
        .file()
        .add_filter("Табличные данные", &["txt", "csv", "tsv", "dat"])
        .add_filter("Все файлы", &["*"])
        .blocking_pick_file();

    match file_path {
        Some(path) => {
            let path = path
                .into_path()
                .map_err(|_| "Некорректный путь файла".to_string())?;
            fs::read_to_string(path).map_err(|error| error.to_string())
        }
        None => Err("Отменено".into()),
    }
}

#[tauri::command]
async fn save_file_dialog(
    app: tauri::AppHandle,
    content: String,
    default_name: Option<String>,
    filter_name: Option<String>,
    filter_ext: Option<String>,
) -> Result<(), String> {
    let mut dialog = app.dialog().file();

    if let Some(name) = default_name {
        dialog = dialog.set_file_name(name);
    }

    if let (Some(name), Some(ext)) = (filter_name, filter_ext) {
        dialog = dialog.add_filter(name, &[ext.as_str()]);
    }

    match dialog.blocking_save_file() {
        Some(path) => {
            let path = path
                .into_path()
                .map_err(|_| "Некорректный путь файла".to_string())?;
            fs::write(path, content).map_err(|error| error.to_string())
        }
        None => Err("Отменено".into()),
    }
}

#[tauri::command]
async fn save_plot_dialog(app: tauri::AppHandle, b64: String) -> Result<(), String> {
    match app
        .dialog()
        .file()
        .set_file_name("if97_plot.png")
        .add_filter("PNG-изображение", &["png"])
        .blocking_save_file()
    {
        Some(path) => {
            let path = path
                .into_path()
                .map_err(|_| "Некорректный путь файла".to_string())?;
            let image_bytes = STANDARD.decode(&b64).map_err(|error| error.to_string())?;
            fs::write(path, image_bytes).map_err(|error| error.to_string())
        }
        None => Err("Отменено".into()),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
/// Запускает Tauri-приложение (desktop/mobile).
pub fn run() {
    init_logging();
    clear_logs_buffer();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            read_logs,
            clear_logs,
            calculate_single,
            calculate_table,
            calculate_dome,
            load_file_dialog,
            save_file_dialog,
            save_plot_dialog
        ])
        .run(tauri::generate_context!())
        .expect("ошибка при запуске приложения Tauri");
}

#[cfg(test)]
mod dome_perf_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    #[ignore]
    fn dome_perf_all_diagrams() {
        let diagrams = [
            DiagramKind::Pt,
            DiagramKind::Pv,
            DiagramKind::Tv,
            DiagramKind::Ph,
            DiagramKind::Ps,
            DiagramKind::Ts,
            DiagramKind::Th,
            DiagramKind::Hs,
        ];

        for &chart_type in &diagrams {
            for &swap_axes in &[false, true] {
                let start = Instant::now();
                let points = calculate_dome_points(DomeRequest {
                    chart_type,
                    swap_axes,
                });
                let elapsed = start.elapsed().as_secs_f64();
                println!(
                    "dome_perf: chart={:?} swap_axes={} points={} elapsed={:.3}s",
                    chart_type,
                    swap_axes,
                    points.len(),
                    elapsed
                );
            }
        }
    }
}

#[cfg(test)]
mod smoke_tests {
    use super::*;

    #[test]
    fn calculate_state_smoke_pt_ok() {
        let state = calculate_state(SingleCalcRequest {
            mode: InputMode::Pt,
            v1: 0.1,
            v2: 300.0,
        })
        .expect("calculate_state pt");

        assert!(state.h.is_finite());
        assert!(state.s.is_finite());
        assert!(!state.region.trim().is_empty());
    }

    #[test]
    fn calculate_table_rows_smoke_parses_mixed_separators() {
        let input = "0.1 300\n0.1;300\n0,1\t300\n\n".to_string();
        let rows = calculate_table_rows(TableCalcRequest {
            mode: InputMode::Pt,
            input,
        });

        assert_eq!(rows.len(), 3);
        for row in rows {
            assert!(row.error.is_none());
            let state = row.state.expect("state");
            assert!(state.h.is_finite());
        }
    }

    #[test]
    fn calculate_dome_points_smoke_non_empty() {
        let points = calculate_dome_points(DomeRequest {
            chart_type: DiagramKind::Pt,
            swap_axes: false,
        });
        assert!(!points.is_empty());
    }
}
