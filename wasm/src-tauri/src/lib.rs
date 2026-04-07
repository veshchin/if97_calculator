use base64::{engine::general_purpose::STANDARD, Engine as _};
use if97_app_api::{
    DiagramKind, DomeRequest, InputMode, LogEntryDto, PlotPoint, SingleCalcRequest, StateDto,
    TableCalcRequest, TableRowResult,
};
use if97_core::{errors::If97Error, If97, WaterState};
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

fn calculate_dome_points(request: DomeRequest) -> Vec<PlotPoint> {
    let mut liquid = Vec::new();
    let mut vapor = Vec::new();
    let p_min = 0.000611657_f64;
    let p_max = 22.064_f64;
    let steps = 200;

    for index in 0..=steps {
        let t = index as f64 / steps as f64;
        let pressure = (p_min.ln() + t * (p_max.ln() - p_min.ln())).exp();
        if let Ok(state) = If97::px(pressure.into(), 0.0.into()) {
            liquid.push(project_state(request.chart_type, request.swap_axes, state));
        }
        if let Ok(state) = If97::px(pressure.into(), 1.0.into()) {
            vapor.push(project_state(request.chart_type, request.swap_axes, state));
        }
    }

    vapor.reverse();
    liquid.extend(vapor);
    liquid
}

#[tauri::command]
async fn read_logs() -> Result<Vec<LogEntryDto>, String> {
    Ok(snapshot_logs())
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
pub fn run() {
    init_logging();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            read_logs,
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
