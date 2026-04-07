// File: src/state.rs

use if97_core::WaterState;
use tracing::info;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PlotType {
    PT,
    RhoT,
    VT,
}

#[derive(Clone)]
pub struct SavedData {
    pub name: String,
    pub points: Vec<WaterState>,
    pub visible: bool,
}

pub struct AppState {
    pub datasets: Vec<SavedData>,
    pub last_single_result: Option<WaterState>,
    pub plot_type: PlotType,
    pub show_dome: bool,
    pub swap_axes: bool,
    pub autoscale: bool,
    pub custom_limits: (f64, f64, f64, f64),
}

impl AppState {
    pub fn new() -> Self {
        info!("Инициализация AppState с настройками по умолчанию");
        Self {
            datasets: vec![SavedData {
                name: "Текущая таблица".to_string(),
                points: vec![],
                visible: true,
            }],
            last_single_result: None,
            plot_type: PlotType::PT,
            show_dome: true,
            swap_axes: false,
            autoscale: true,
            custom_limits: (0.0, 100.0, 273.15, 1000.0),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    // Вкладка 1: Одиночный расчет
    CalculateSingle { mode: i32, val_a: f64, val_b: f64 },
    SaveSinglePoint,

    // Вкладка 2: Табличный расчет
    LoadBatchFile,
    SaveBatchTable,
    BatchDataChanged { mode: i32, content: String },

    // Вкладка 3: Графики
    ChangePlotType(PlotType),
    ToggleDome(bool),
    ToggleSwapAxes(bool),
    SetAutoscale(bool),
    ApplyPlotLimits(f64, f64, f64, f64),
    SelectData,
    UpdateDataVisibility(Vec<bool>),
    ExportPlot,

    // Экспорт
    SaveLogFile,
    ExportBatchData,
}
