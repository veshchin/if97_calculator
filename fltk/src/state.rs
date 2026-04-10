//! Состояние приложения и сообщения UI для FLTK-клиента.

use if97_app_api::{AxisVar, InputMode};
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
    /// Величина по оси X на вкладке графика.
    pub plot_x: AxisVar,
    /// Величина по оси Y на вкладке графика.
    pub plot_y: AxisVar,
    /// Показывать купол насыщения.
    pub show_dome: bool,
    /// Логарифмическая шкала по оси X.
    pub plot_x_log: bool,
    /// Логарифмическая шкала по оси Y.
    pub plot_y_log: bool,
    /// Автоматический подбор пределов осей.
    pub autoscale: bool,
    /// Пользовательские пределы осей (xmin, xmax, ymin, ymax).
    pub custom_limits: (f64, f64, f64, f64),
    /// Количество знаков после запятой в одиночном расчете.
    pub single_precision: usize,
    /// Количество знаков после запятой в табличном расчете.
    pub table_precision: usize,
    /// Использовать научный формат чисел в табличном выводе (например, `1.0694e3`).
    pub table_scientific: bool,
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
            plot_x: AxisVar::T,
            plot_y: AxisVar::P,
            show_dome: true,
            plot_x_log: false,
            plot_y_log: false,
            autoscale: true,
            custom_limits: (273.15, 1000.0, 0.001, 100.0),
            single_precision: 4,
            table_precision: 4,
            table_scientific: false,
        }
    }

    /// Возвращает изменяемую ссылку на dataset "Текущая таблица".
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
    /// Выполнить одиночный расчет для пары значений.
    CalculateSingle {
        /// Режим расчета.
        mode: InputMode,
        /// Первое значение (сырой текст из поля ввода).
        raw_a: String,
        /// Второе значение (сырой текст из поля ввода).
        raw_b: String,
    },
    /// Установить точность вывода (знаков после запятой) для одиночного расчета.
    SetSinglePrecision(usize),
    /// Сохранить (или обновить) текущую одиночную точку в списке данных.
    SaveSinglePoint {
        /// Имя сохраненной точки.
        name: String,
        /// Режим расчета.
        mode: InputMode,
        /// Первое значение (сырой текст).
        raw_a: String,
        /// Второе значение (сырой текст).
        raw_b: String,
    },
    /// Загрузить сохраненную точку по индексу среди точек.
    LoadSavedPoint(usize),
    /// Удалить сохраненную точку по индексу среди точек.
    DeleteSavedPoint(usize),

    /// Открыть файл с табличными данными.
    LoadBatchFile,
    /// Установить точность вывода (знаков после запятой) для табличного расчета.
    SetTablePrecision(usize),
    /// Включить/выключить научный формат чисел в табличном выводе.
    SetTableScientific(bool),
    /// Сохранить (или обновить) текущую таблицу в списке данных.
    SaveBatchTable {
        /// Имя сохраненной таблицы.
        name: String,
        /// Режим расчета.
        mode: InputMode,
        /// Исходный текст таблицы.
        content: String,
    },
    /// Загрузить сохраненную таблицу по индексу среди таблиц.
    LoadSavedTable(usize),
    /// Удалить сохраненную таблицу по индексу среди таблиц.
    DeleteSavedTable(usize),
    /// Изменение текста таблицы (требует пересчета результата).
    BatchDataChanged {
        /// Режим расчета.
        mode: InputMode,
        /// Новый текст таблицы.
        content: String,
    },
    /// Сгенерировать строки таблицы по диапазонам значений.
    GenerateBatchData {
        /// Начало диапазона первого параметра.
        v1_from: String,
        /// Конец диапазона первого параметра.
        v1_to: String,
        /// Шаг первого параметра.
        v1_step: String,
        /// Начало диапазона второго параметра.
        v2_from: String,
        /// Конец диапазона второго параметра.
        v2_to: String,
        /// Шаг второго параметра.
        v2_step: String,
    },

    /// Установить величину по оси X на вкладке графика.
    SetPlotX(AxisVar),
    /// Установить величину по оси Y на вкладке графика.
    SetPlotY(AxisVar),
    /// Показать/скрыть купол насыщения.
    ToggleDome(bool),
    /// Включить/выключить логарифмическую шкалу по оси X.
    TogglePlotXLog(bool),
    /// Включить/выключить логарифмическую шкалу по оси Y.
    TogglePlotYLog(bool),
    /// Включить/выключить автомасштаб осей.
    SetAutoscale(bool),
    /// Применить ручные пределы осей (x_min, x_max, y_min, y_max).
    ApplyPlotLimits(f64, f64, f64, f64),
    /// Открыть выбор наборов данных, отображаемых на графике.
    SelectData,
    /// Установить видимость наборов данных (по порядку списка).
    UpdateDataVisibility(Vec<bool>),
    /// Экспортировать график в PNG.
    ExportPlot,

    /// Экспортировать журнал работы приложения в файл.
    SaveLogFile,
    /// Экспортировать табличные данные в CSV.
    ExportBatchData,
}
