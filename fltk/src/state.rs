use if97_app_api::{DiagramKind, InputMode};
use if97_core::WaterState;
use tracing::info;

#[derive(Clone, Debug)]
pub enum SavedKind {
    CurrentTable,
    Point {
        mode: InputMode,
        v1: String,
        v2: String,
    },
    Table {
        mode: InputMode,
        input: String,
    },
}

#[derive(Clone)]
pub struct SavedData {
    pub name: String,
    pub points: Vec<WaterState>,
    pub visible: bool,
    pub kind: SavedKind,
}

#[derive(Clone)]
pub struct BatchRow {
    pub line_no: usize,
    pub state: Option<WaterState>,
    pub error: Option<String>,
}

pub struct AppState {
    pub datasets: Vec<SavedData>,
    pub current_table_rows: Vec<BatchRow>,
    pub last_single_result: Option<WaterState>,
    pub last_single_request: Option<(InputMode, String, String)>,
    pub plot_type: DiagramKind,
    pub show_dome: bool,
    pub swap_axes: bool,
    pub autoscale: bool,
    pub custom_limits: (f64, f64, f64, f64),
    pub single_precision: usize,
    pub table_precision: usize,
}

impl AppState {
    pub fn new() -> Self {
        info!("Инициализация AppState с настройками по умолчанию");
        Self {
            datasets: vec![SavedData {
                name: "Текущая таблица".to_string(),
                points: vec![],
                visible: true,
                kind: SavedKind::CurrentTable,
            }],
            current_table_rows: Vec::new(),
            last_single_result: None,
            last_single_request: None,
            plot_type: DiagramKind::Pt,
            show_dome: true,
            swap_axes: false,
            autoscale: true,
            custom_limits: (273.15, 1000.0, 0.001, 100.0),
            single_precision: 4,
            table_precision: 4,
        }
    }

    pub fn current_table_dataset_mut(&mut self) -> &mut SavedData {
        &mut self.datasets[0]
    }

    pub fn saved_point_labels(&self) -> Vec<String> {
        self.datasets
            .iter()
            .filter_map(|dataset| match dataset.kind {
                SavedKind::Point { .. } => Some(dataset.name.clone()),
                _ => None,
            })
            .collect()
    }

    pub fn saved_table_labels(&self) -> Vec<String> {
        self.datasets
            .iter()
            .filter_map(|dataset| match dataset.kind {
                SavedKind::Table { .. } => Some(dataset.name.clone()),
                _ => None,
            })
            .collect()
    }

    pub fn saved_point_at(&self, index: usize) -> Option<&SavedData> {
        self.datasets
            .iter()
            .filter(|dataset| matches!(dataset.kind, SavedKind::Point { .. }))
            .nth(index)
    }

    pub fn saved_table_at(&self, index: usize) -> Option<&SavedData> {
        self.datasets
            .iter()
            .filter(|dataset| matches!(dataset.kind, SavedKind::Table { .. }))
            .nth(index)
    }

    pub fn remove_saved_point(&mut self, index: usize) {
        if let Some(target_name) = self.saved_point_at(index).map(|dataset| dataset.name.clone()) {
            self.datasets
                .retain(|dataset| !matches!(dataset.kind, SavedKind::Point { .. }) || dataset.name != target_name);
        }
    }

    pub fn remove_saved_table(&mut self, index: usize) {
        if let Some(target_name) = self.saved_table_at(index).map(|dataset| dataset.name.clone()) {
            self.datasets
                .retain(|dataset| !matches!(dataset.kind, SavedKind::Table { .. }) || dataset.name != target_name);
        }
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    CalculateSingle {
        mode: InputMode,
        raw_a: String,
        raw_b: String,
    },
    SetSinglePrecision(usize),
    SaveSinglePoint {
        name: String,
        mode: InputMode,
        raw_a: String,
        raw_b: String,
    },
    LoadSavedPoint(usize),
    DeleteSavedPoint(usize),

    LoadBatchFile,
    SetTablePrecision(usize),
    SaveBatchTable {
        name: String,
        mode: InputMode,
        content: String,
    },
    LoadSavedTable(usize),
    DeleteSavedTable(usize),
    BatchDataChanged {
        mode: InputMode,
        content: String,
    },
    GenerateBatchData {
        v1_from: String,
        v1_to: String,
        v1_step: String,
        v2_from: String,
        v2_to: String,
        v2_step: String,
    },

    ChangePlotType(DiagramKind),
    ToggleDome(bool),
    ToggleSwapAxes(bool),
    SetAutoscale(bool),
    ApplyPlotLimits(f64, f64, f64, f64),
    SelectData,
    UpdateDataVisibility(Vec<bool>),
    ExportPlot,

    SaveLogFile,
    ExportBatchData,
}
