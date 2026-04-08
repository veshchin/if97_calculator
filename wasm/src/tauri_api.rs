use if97_app_api::{
    DomeRequest, LogEntryDto, PlotPoint, SingleCalcRequest, StateDto, TableCalcRequest,
    TableRowResult,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[derive(Serialize)]
struct CommandArgs<T> {
    request: T,
}

#[derive(Serialize)]
pub struct SaveFileArgs {
    pub content: String,
    pub default_name: Option<String>,
    pub filter_name: Option<String>,
    pub filter_ext: Option<String>,
}

#[derive(Serialize)]
struct SavePlotArgs {
    b64: String,
}

#[derive(Serialize)]
struct OpenPathArgs {
    path: String,
}

fn js_error_to_string(error: JsValue) -> String {
    error
        .as_string()
        .unwrap_or_else(|| "Неизвестная ошибка Tauri API".to_string())
}

async fn invoke_command<A, R>(cmd: &str, args: &A) -> Result<R, String>
where
    A: Serialize,
    R: DeserializeOwned,
{
    let js_args = serde_wasm_bindgen::to_value(args).map_err(|error| error.to_string())?;
    let result = invoke(cmd, js_args).await.map_err(js_error_to_string)?;
    serde_wasm_bindgen::from_value(result).map_err(|error| error.to_string())
}

pub async fn calculate_single(request: SingleCalcRequest) -> Result<StateDto, String> {
    invoke_command("calculate_single", &CommandArgs { request }).await
}

pub async fn calculate_table(request: TableCalcRequest) -> Result<Vec<TableRowResult>, String> {
    invoke_command("calculate_table", &CommandArgs { request }).await
}

pub async fn calculate_dome(request: DomeRequest) -> Result<Vec<PlotPoint>, String> {
    invoke_command("calculate_dome", &CommandArgs { request }).await
}

pub async fn load_file_dialog() -> Result<String, String> {
    #[derive(Serialize)]
    struct EmptyArgs {}

    invoke_command("load_file_dialog", &EmptyArgs {}).await
}

pub async fn save_file_dialog(args: SaveFileArgs) -> Result<(), String> {
    invoke_command("save_file_dialog", &args).await
}

pub async fn save_plot_dialog(b64: String) -> Result<(), String> {
    invoke_command("save_plot_dialog", &SavePlotArgs { b64 }).await
}

pub async fn open_external(path: String) -> Result<(), String> {
    invoke_command("plugin:shell|open", &OpenPathArgs { path }).await
}

pub async fn read_logs() -> Result<Vec<LogEntryDto>, String> {
    #[derive(Serialize)]
    struct EmptyArgs {}

    invoke_command("read_logs", &EmptyArgs {}).await
}

pub async fn clear_logs() -> Result<(), String> {
    #[derive(Serialize)]
    struct EmptyArgs {}

    invoke_command("clear_logs", &EmptyArgs {}).await
}
