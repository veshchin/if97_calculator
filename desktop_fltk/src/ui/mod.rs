// desktop_fltk/src/ui/mod.rs

pub mod single;
pub mod batch;
pub mod plot_tab;

use fltk::{prelude::*, window::Window, group::Tabs};
use fltk::app::Sender;
use crate::state::Message;
use single::SingleTab;
use batch::BatchTab;
use plot_tab::PlotTab;

pub struct MainUI {
    pub window: Window,
    pub single_tab: SingleTab,
    pub batch_tab: BatchTab,
    pub plot_tab: PlotTab,
}

impl MainUI {
    pub fn new(sender: Sender<Message>) -> Self {
        let mut window = Window::default().with_size(1050, 700).with_label("IAPWS-IF97 Calculator Pro");
        window.make_resizable(true);

        let tabs = Tabs::new(10, 10, 1030, 680, "");

        let single_tab = SingleTab::new(sender.clone());
        let batch_tab = BatchTab::new(sender.clone());
        let plot_tab = PlotTab::new(sender);

        tabs.end();
        window.end();

        Self { window, single_tab, batch_tab, plot_tab }
    }
}