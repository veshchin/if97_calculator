/* File: src/types.rs */
use if97_core::WaterState;
use yew::UseStateHandle;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq)]
pub struct SavedPoint {
    pub name: String,
    pub state: WaterState,
    pub orig_mode: String,
    pub orig_v1: String,
    pub orig_v2: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SavedTable {
    pub name: String,
    pub states: Vec<WaterState>,
    pub orig_mode: String,
    pub orig_input: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SavedItem {
    Point(SavedPoint),
    Table(SavedTable),
}

pub type AppContext = UseStateHandle<Vec<SavedItem>>;

#[derive(Clone, Debug, PartialEq)]
pub struct ThermodynamicPoint {
    pub p: f64,
    pub t: f64,
    pub h: f64,
    pub s: f64,
    pub v: f64,
    pub x: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ChartType { Ts, Hs, Ph, Tv, Pv, Pt, Ps, Th }

#[derive(Clone, Debug, PartialEq)]
pub struct PersistentState {
    pub s_mode: String,
    pub s_v1: String,
    pub s_v2: String,
    pub s_res: Option<WaterState>,
    pub s_error: Option<String>,
    pub t_input: String,
    pub t_mode: String,
    pub t_res: Vec<Result<WaterState, String>>,
    pub right_sidebar_open: bool,
    pub plot_selected: HashSet<String>,
    pub is_dark_theme: bool, // Поле logs удалено
}

pub type StateContext = UseStateHandle<PersistentState>;