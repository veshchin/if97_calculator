//! Состояние приложения и сообщения UI для FLTK-клиента.

use if97_app_api::{DiagramKind, InputMode};
use if97_core::WaterState;
use tracing::info;

#[derive(Clone, Debug)]
/// Тип сохраненного набора данных (точка или таблица), используемый для UI и построения графиков.
pub enum SavedKind {
    /// Текущая таблица (виртуальный dataset, который не сохраняется как отдельный элемент).
    CurrentTable,
    /// Одна рассчитанная точка, сохраненная пользователем.
    Point {
        /// Режим расчета.
        mode: InputMode,
        /// Первое значение, как было введено пользователем.
        v1: String,
        /// Второе значение, как было введено пользователем.
        v2: String,
    },
    /// Табличный расчет, сохраненный пользователем.
    Table {
        /// Режим расчета.
        mode: InputMode,
        /// Исходный текст табличного ввода.
        input: String,
    },
}

#[derive(Clone)]
/// Сохраненный набор точек (для вкладок "Таблица"/"График").
pub struct SavedData {
    /// Отображаемое имя набора.
    pub name: String,
    /// Набор рассчитанных состояний.
    pub points: Vec<WaterState>,
    /// Флаг видимости набора на графике.
    pub visible: bool,
    /// Тип набора (точка/таблица/текущая таблица).
    pub kind: SavedKind,
}

#[derive(Clone)]
/// Результат обработки одной строки табличного ввода.
pub struct BatchRow {
    /// Номер строки во входном тексте (1-based).
    pub line_no: usize,
    /// Рассчитанное состояние (если расчет успешен).
    pub state: Option<WaterState>,
    /// Ошибка строки (если парсинг или расчет не удались).
    pub error: Option<String>,
}

/// Глобальное состояние приложения.
pub struct AppState {
    /// Наборы данных (включая "Текущая таблица" как первый элемент).
    pub datasets: Vec<SavedData>,
    /// Результаты расчета текущей таблицы (по строкам).
    pub current_table_rows: Vec<BatchRow>,
    /// Последний успешный результат одиночного расчета.
    pub last_single_result: Option<WaterState>,
    /// Параметры последнего одиночного расчета (для восстановления UI).
    pub last_single_request: Option<(InputMode, String, String)>,
    /// Тип диаграммы для вкладки графика.
    pub plot_type: DiagramKind,
    /// Показывать купол насыщения.
    pub show_dome: bool,
    /// Поменять оси местами.
    pub swap_axes: bool,
    /// Автоматический подбор пределов осей.
    pub autoscale: bool,
    /// Пользовательские пределы осей (xmin, xmax, ymin, ymax).
    pub custom_limits: (f64, f64, f64, f64),
    /// Количество знаков после запятой в одиночном расчете.
    pub single_precision: usize,
    /// Количество знаков после запятой в табличном расчете.
    pub table_precision: usize,
}

impl AppState {
    /// Создает состояние приложения с дефолтными настройками.
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

    /// Возвращает список имен сохраненных одиночных точек.
    pub fn saved_point_labels(&self) -> Vec<String> {
        self.datasets
            .iter()
            .filter_map(|dataset| match dataset.kind {
                SavedKind::Point { .. } => Some(dataset.name.clone()),
                _ => None,
            })
            .collect()
    }

    /// Возвращает список имен сохраненных таблиц.
    pub fn saved_table_labels(&self) -> Vec<String> {
        self.datasets
            .iter()
            .filter_map(|dataset| match dataset.kind {
                SavedKind::Table { .. } => Some(dataset.name.clone()),
                _ => None,
            })
            .collect()
    }

    /// Возвращает сохраненную точку по индексу среди точек.
    pub fn saved_point_at(&self, index: usize) -> Option<&SavedData> {
        self.datasets
            .iter()
            .filter(|dataset| matches!(dataset.kind, SavedKind::Point { .. }))
            .nth(index)
    }

    /// Возвращает сохраненную таблицу по индексу среди таблиц.
    pub fn saved_table_at(&self, index: usize) -> Option<&SavedData> {
        self.datasets
            .iter()
            .filter(|dataset| matches!(dataset.kind, SavedKind::Table { .. }))
            .nth(index)
    }

    /// Удаляет сохраненную точку по индексу среди точек.
    pub fn remove_saved_point(&mut self, index: usize) {
        if let Some(target_name) = self.saved_point_at(index).map(|dataset| dataset.name.clone()) {
            self.datasets
                .retain(|dataset| !matches!(dataset.kind, SavedKind::Point { .. }) || dataset.name != target_name);
        }
    }

    /// Удаляет сохраненную таблицу по индексу среди таблиц.
    pub fn remove_saved_table(&mut self, index: usize) {
        if let Some(target_name) = self.saved_table_at(index).map(|dataset| dataset.name.clone()) {
            self.datasets
                .retain(|dataset| !matches!(dataset.kind, SavedKind::Table { .. }) || dataset.name != target_name);
        }
    }
}

#[derive(Clone, Debug)]
/// Сообщения, которыми UI управляет состоянием и обработчиками действий.
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
