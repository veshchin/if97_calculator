// File: src/ui/mod.rs

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
pub struct MainUI {
    pub window: Window,
    pub single_tab: SingleTab,
    pub batch_tab: BatchTab,
    pub plot_tab: PlotTab,
    pub about_tab: AboutTab,
}

impl MainUI {
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
