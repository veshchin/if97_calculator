//! UI-компоненты (вкладки) для Yew-frontend.

/// Вкладка "О программе" и отображение журнала.
pub mod about_logs;
/// Вкладка построения диаграмм.
pub mod plots;
/// Вкладка одиночного расчета.
pub mod single_calc;
/// Вкладка табличного расчета.
pub mod table_calc;

/// Компонент вкладки "О программе" и журнала.
pub use about_logs::AboutLogsTab;
/// Компонент вкладки диаграмм.
pub use plots::PlotsTab;
/// Компонент вкладки одиночного расчета.
pub use single_calc::SingleCalcTab;
/// Компонент вкладки табличного расчета.
pub use table_calc::TableCalcTab;
