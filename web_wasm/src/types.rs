use if97_core::domain::state::WaterState;
use web_sys::WheelEvent;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PlotType {
    PT,
    PV,
    TS,
    HS,
    PH,
    RhoT,
    VT,
}
#[derive(Clone)]
pub struct SavedData {
    pub name: String,
    pub points: Vec<WaterState>,
    pub visible: bool,
}

pub enum Msg {
    SwitchTab(usize),

    // Одиночный расчет
    SetSingleMode(i32),
    UpdateInputA(String),
    UpdateInputB(String),
    CalculateSingle,
    SaveSinglePoint,

    // Табличный расчет
    SetBatchMode(i32),
    UpdateBatchInput(String),
    CalculateBatch,
    SaveBatchTable,

    // Графики
    SetPlotType(PlotType),
    ToggleDome(bool),
    ToggleSwapAxes(bool),
    ToggleAutoscale(bool),
    UpdateLimit(u8, String),
    ToggleDatasetVisibility(usize, bool),

    // Навигация по графику
    ZoomPlot(WheelEvent),
    PlotMouseDown(i32, i32),
    PlotMouseMove(i32, i32),
    PlotMouseUp,

    UpdatePrecision(usize),
}