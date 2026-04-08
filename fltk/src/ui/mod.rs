//! Компоненты UI для FLTK-приложения.

pub mod about;
pub mod batch;
pub mod plot_tab;
pub mod single;

use crate::state::Message;
use about::AboutTab;
use batch::BatchTab;
use fltk::app::Sender;
use fltk::{group::Tabs, prelude::*, window::Window};
use plot_tab::PlotTab;
use single::SingleTab;

#[allow(dead_code)]
/// Главные виджеты приложения (окно и вкладки).
pub struct MainUI {
    /// Главное окно приложения.
    pub window: Window,
    /// Вкладка одиночного расчета.
    pub single_tab: SingleTab,
    /// Вкладка табличного расчета.
    pub batch_tab: BatchTab,
    /// Вкладка построения графиков.
    pub plot_tab: PlotTab,
    /// Вкладка "О программе" и журнал.
    pub about_tab: AboutTab,
}

impl MainUI {
    /// Собирает UI и возвращает структуру с основными виджетами.
    pub fn new(sender: Sender<Message>) -> Self {
        let mut window = Window::default()
            .with_size(1050, 700)
            .with_label("IF97 Calculator");
        window.make_resizable(true);

        let tabs = Tabs::new(10, 10, 1030, 680, "");
        let single_tab = SingleTab::new(sender.clone());
        let batch_tab = BatchTab::new(sender.clone());
        let plot_tab = PlotTab::new(sender.clone());
        let about_tab = AboutTab::new(sender);

        tabs.end();
        window.end();

        Self {
            window,
            single_tab,
            batch_tab,
            plot_tab,
            about_tab,
        }
    }
}
