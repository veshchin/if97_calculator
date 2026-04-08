use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use if97_app_api::{
    DiagramKind, DomeRequest, PlotPoint, SingleCalcRequest, StateDto, TableCalcRequest,
    TableRowResult,
};
use if97_core::{errors::If97Error, If97, WaterState};
use std::net::SocketAddr;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
struct AppState {
    version: &'static str,
}

#[derive(Debug, serde::Serialize)]
struct ServiceInfo {
    name: &'static str,
    version: &'static str,
    status: &'static str,
}

#[derive(Debug, serde::Serialize)]
struct ErrorBody {
    error: String,
}

struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(ErrorBody { error: self.message })).into_response()
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
        if97_app_api::InputMode::Pt => If97::pt(request.v1.into(), request.v2.into()),
        if97_app_api::InputMode::Ph => If97::ph(request.v1.into(), request.v2.into()),
        if97_app_api::InputMode::Ps => If97::ps(request.v1.into(), request.v2.into()),
        if97_app_api::InputMode::Px => If97::px(request.v1.into(), request.v2.into()),
        if97_app_api::InputMode::Rhot => If97::rhot(request.v1.into(), request.v2.into()),
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

const SAT_P_MIN: f64 = 0.000611657_f64;
const SAT_P_MAX: f64 = 22.064_f64;
const SAT_MAX_DEPTH: usize = 12;
const SAT_REL_TOL: f64 = 0.0025;

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
    let state = If97::px(pressure.into(), quality.into()).ok()?;
    Some(project_state(chart_type, swap_axes, state))
}

fn point_line_distance(a: PlotPoint, b: PlotPoint, m: PlotPoint) -> f64 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let denom = (dx * dx + dy * dy).sqrt();
    if denom <= f64::EPSILON {
        return 0.0;
    }
    let cross = (dx * (m.y - a.y) - dy * (m.x - a.x)).abs();
    cross / denom
}

fn refine_saturation_segment(
    chart_type: DiagramKind,
    swap_axes: bool,
    quality: f64,
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

    let seg_len = ((p1.x - p0.x).powi(2) + (p1.y - p0.y).powi(2)).sqrt();
    let deviation = point_line_distance(p0, p1, pm);
    if seg_len > 0.0 && deviation > seg_len * SAT_REL_TOL {
        refine_saturation_segment(chart_type, swap_axes, quality, t0, p0, mid_t, pm, depth + 1, out);
        refine_saturation_segment(chart_type, swap_axes, quality, mid_t, pm, t1, p1, depth + 1, out);
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

    let mut points = Vec::new();
    refine_saturation_segment(chart_type, swap_axes, quality, 0.0, p0, 1.0, p1, 0, &mut points);
    points.push(p1);
    points
}

fn calculate_dome_points(request: DomeRequest) -> Vec<PlotPoint> {
    let liquid = build_saturation_side(request.chart_type, request.swap_axes, 0.0);
    if liquid.is_empty() {
        return Vec::new();
    }

    if request.chart_type == DiagramKind::Pt {
        return liquid;
    }

    let mut vapor = build_saturation_side(request.chart_type, request.swap_axes, 1.0);
    vapor.reverse();

    let mut dome = liquid;
    dome.extend(vapor);
    dome
}

async fn health(State(state): State<AppState>) -> Json<ServiceInfo> {
    Json(ServiceInfo {
        name: "if97_calculator_service",
        version: state.version,
        status: "ok",
    })
}

async fn calculate_single_handler(
    Json(request): Json<SingleCalcRequest>,
) -> Result<Json<StateDto>, ApiError> {
    let result = tokio::task::spawn_blocking(move || calculate_state(request))
        .await
        .map_err(|err| ApiError::internal(err.to_string()))?;

    result
        .map(Json)
        .map_err(ApiError::bad_request)
}

async fn calculate_table_handler(
    Json(request): Json<TableCalcRequest>,
) -> Result<Json<Vec<TableRowResult>>, ApiError> {
    let rows = tokio::task::spawn_blocking(move || calculate_table_rows(request))
        .await
        .map_err(|err| ApiError::internal(err.to_string()))?;
    Ok(Json(rows))
}

async fn calculate_dome_handler(
    Json(request): Json<DomeRequest>,
) -> Result<Json<Vec<PlotPoint>>, ApiError> {
    let points = tokio::task::spawn_blocking(move || calculate_dome_points(request))
        .await
        .map_err(|err| ApiError::internal(err.to_string()))?;
    Ok(Json(points))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,tower_http=info,if97_calculator_service=debug")),
        )
        .init();

    let bind_addr = std::env::var("IF97_SERVICE_ADDR")
        .ok()
        .and_then(|raw| raw.parse::<SocketAddr>().ok())
        .unwrap_or_else(|| SocketAddr::from(([0, 0, 0, 0], 8080)));

    let app = Router::new()
        .route("/healthz", get(health))
        .route("/api/v1/calculate/single", post(calculate_single_handler))
        .route("/api/v1/calculate/table", post(calculate_table_handler))
        .route("/api/v1/plot/dome", post(calculate_dome_handler))
        .layer(CompressionLayer::new())
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .layer(TraceLayer::new_for_http())
        .with_state(AppState {
            version: env!("CARGO_PKG_VERSION"),
        });

    tracing::info!("service listening on {bind_addr}");
    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
