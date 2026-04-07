use if97_app_api::{DiagramKind, InputMode, StateDto, TableRowResult};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use yew::UseStateHandle;

pub type SavedId = u64;

#[derive(Clone, Debug, PartialEq)]
pub struct SavedPoint {
    pub id: SavedId,
    pub name: String,
    pub state: StateDto,
    pub orig_mode: InputMode,
    pub orig_v1: String,
    pub orig_v2: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SavedTable {
    pub id: SavedId,
    pub name: String,
    pub states: Vec<StateDto>,
    pub orig_mode: InputMode,
    pub orig_input: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SavedItem {
    Point(SavedPoint),
    Table(SavedTable),
}

impl SavedItem {
    pub fn id(&self) -> SavedId {
        match self {
            Self::Point(point) => point.id,
            Self::Table(table) => table.id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Point(point) => &point.name,
            Self::Table(table) => &table.name,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SavedItemsState {
    pub items: Vec<SavedItem>,
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
    pub fn allocate_id(&mut self) -> SavedId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

pub type AppContext = UseStateHandle<SavedItemsState>;
pub type ChartType = DiagramKind;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct GenParams {
    pub v1_from: String,
    pub v1_to: String,
    pub v1_step: String,
    pub v2_from: String,
    pub v2_to: String,
    pub v2_step: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PersistentState {
    pub s_mode: InputMode,
    pub s_v1: String,
    pub s_v2: String,
    pub s_res: Option<StateDto>,
    pub s_error: Option<String>,
    pub s_precision: usize,
    pub t_input: String,
    pub t_mode: InputMode,
    pub t_res: Rc<Vec<TableRowResult>>,
    pub t_precision: usize,
    pub t_gen_params: HashMap<InputMode, GenParams>,
    pub right_sidebar_open: bool,
    pub plot_selected: HashSet<SavedId>,
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

pub type StateContext = UseStateHandle<PersistentState>;
