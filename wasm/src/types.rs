//! Типы состояния UI для web/Tauri-frontend.
//!
//! Эти структуры используются внутри Yew и хранят как текущие значения полей ввода,
//! так и сохраненные пользователем результаты расчета.

use if97_app_api::{DiagramKind, InputMode, StateDto, TableRowResult};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use yew::UseStateHandle;

/// Идентификатор сохраненного элемента (точка/таблица).
pub type SavedId = u64;

#[derive(Clone, Debug, PartialEq)]
/// Сохраненная пользователем точка расчета.
pub struct SavedPoint {
    /// Идентификатор элемента.
    pub id: SavedId,
    /// Отображаемое имя.
    pub name: String,
    /// Результат расчета.
    pub state: StateDto,
    /// Режим, с которым была рассчитана точка.
    pub orig_mode: InputMode,
    /// Первое значение, как было введено пользователем.
    pub orig_v1: String,
    /// Второе значение, как было введено пользователем.
    pub orig_v2: String,
}

#[derive(Clone, Debug, PartialEq)]
/// Сохраненная пользователем таблица расчета.
pub struct SavedTable {
    /// Идентификатор элемента.
    pub id: SavedId,
    /// Отображаемое имя.
    pub name: String,
    /// Список рассчитанных состояний.
    pub states: Vec<StateDto>,
    /// Режим, с которым была рассчитана таблица.
    pub orig_mode: InputMode,
    /// Исходный текст табличного ввода.
    pub orig_input: String,
}

#[derive(Clone, Debug, PartialEq)]
/// Сохраненный элемент данных (точка или таблица).
pub enum SavedItem {
    /// Одиночная точка.
    Point(SavedPoint),
    /// Таблица.
    Table(SavedTable),
}

impl SavedItem {
    /// Возвращает идентификатор элемента.
    pub fn id(&self) -> SavedId {
        match self {
            Self::Point(point) => point.id,
            Self::Table(table) => table.id,
        }
    }

    /// Возвращает отображаемое имя элемента.
    pub fn name(&self) -> &str {
        match self {
            Self::Point(point) => &point.name,
            Self::Table(table) => &table.name,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
/// Состояние списка сохраненных элементов.
pub struct SavedItemsState {
    /// Список элементов.
    pub items: Vec<SavedItem>,
    /// Следующий id для `allocate_id`.
    pub next_id: SavedId,
}

impl Default for SavedItemsState {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            next_id: 1,
        }
    }
}

impl SavedItemsState {
    /// Выделяет новый `SavedId`.
    pub fn allocate_id(&mut self) -> SavedId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

/// Контекст Yew для доступа к сохраненным данным.
pub type AppContext = UseStateHandle<SavedItemsState>;
/// Тип диаграммы (используется в графиках).
pub type ChartType = DiagramKind;

#[derive(Clone, Debug, PartialEq, Default)]
/// Параметры генерации строк для табличного ввода.
pub struct GenParams {
    /// Начало диапазона первого параметра.
    pub v1_from: String,
    /// Конец диапазона первого параметра.
    pub v1_to: String,
    /// Шаг первого параметра.
    pub v1_step: String,
    /// Начало диапазона второго параметра.
    pub v2_from: String,
    /// Конец диапазона второго параметра.
    pub v2_to: String,
    /// Шаг второго параметра.
    pub v2_step: String,
}

#[derive(Clone, Debug, PartialEq)]
/// Состояние, которое сохраняется при переключении вкладок и используется как источник правды для UI.
pub struct PersistentState {
    /// Режим одиночного расчета.
    pub s_mode: InputMode,
    /// Первое значение одиночного расчета (как строка).
    pub s_v1: String,
    /// Второе значение одиночного расчета (как строка).
    pub s_v2: String,
    /// Последний успешный результат одиночного расчета.
    pub s_res: Option<StateDto>,
    /// Ошибка одиночного расчета (если есть).
    pub s_error: Option<String>,
    /// Точность вывода в одиночном расчете.
    pub s_precision: usize,
    /// Текст табличного ввода.
    pub t_input: String,
    /// Режим табличного расчета.
    pub t_mode: InputMode,
    /// Результаты табличного расчета (по строкам).
    pub t_res: Rc<Vec<TableRowResult>>,
    /// Точность вывода в таблице.
    pub t_precision: usize,
    /// Параметры генерации строк таблицы (по режимам).
    pub t_gen_params: HashMap<InputMode, GenParams>,
    /// Состояние панели сохраненных данных.
    pub right_sidebar_open: bool,
    /// Выбранные для отображения на графике элементы.
    pub plot_selected: HashSet<SavedId>,
    /// Текущая тема UI.
    pub is_dark_theme: bool,
}

impl Default for PersistentState {
    fn default() -> Self {
        Self {
            s_mode: InputMode::Pt,
            s_v1: String::new(),
            s_v2: String::new(),
            s_res: None,
            s_error: None,
            s_precision: 4,
            t_input: String::new(),
            t_mode: InputMode::Pt,
            t_res: Rc::new(Vec::new()),
            t_precision: 4,
            t_gen_params: HashMap::new(),
            right_sidebar_open: false,
            plot_selected: HashSet::new(),
            is_dark_theme: false,
        }
    }
}

/// Контекст Yew для доступа к `PersistentState`.
pub type StateContext = UseStateHandle<PersistentState>;
