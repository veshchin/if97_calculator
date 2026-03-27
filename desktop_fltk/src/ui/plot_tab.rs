// File: src/ui/plot_tab.rs

use fltk::{prelude::*, group::*, button::*, input::*, menu::*, frame::*, enums::*, image::RgbImage};
use fltk::app::Sender;
use crate::state::{Message, PlotType, AppState};
use crate::plot::renderer::render_plot_to_buffer;
use tracing::{info, debug};

pub struct PlotTab {
    pub group: Group,
    pub choice_plot_type: Choice,
    pub plot_frame: Frame,
    pub inp_val_min: Input,
    pub inp_val_max: Input,
    pub inp_t_min: Input,
    pub inp_t_max: Input,
    pub btn_apply_limits: Button,
}

impl PlotTab {
    pub fn new(sender: Sender<Message>) -> Self {
        let group = Group::new(10, 35, 1030, 655, " Графики ");

        let mut choice_plot_type = Choice::new(20, 45, 180, 30, "Тип диаграммы:");
        choice_plot_type.set_align(Align::TopLeft);
        choice_plot_type.add_choice("p-T Диаграмма|rho-T Диаграмма|v-T Диаграмма");
        choice_plot_type.set_value(0);

        let mut btn_select_data = Button::new(210, 45, 140, 30, "Выбрать данные");
        let mut check_dome = CheckButton::new(360, 45, 180, 30, "Показывать купол");
        check_dome.set_value(true);

        let mut check_swap = CheckButton::new(550, 45, 130, 30, "Поменять оси");
        check_swap.set_value(false);

        let mut btn_export_plot = Button::new(690, 45, 150, 30, "Экспорт в PNG");
        btn_export_plot.set_color(Color::from_rgb(100, 149, 237));
        btn_export_plot.set_label_color(Color::White);

        let mut check_autoscale = CheckButton::new(20, 85, 120, 30, "Автомасштаб");
        check_autoscale.set_value(true);

        let mut inp_val_min = Input::new(250, 85, 60, 30, "p, МПа от:");
        let mut inp_val_max = Input::new(350, 85, 60, 30, "до:");
        let mut inp_t_min = Input::new(480, 85, 60, 30, "T, К от:");
        let mut inp_t_max = Input::new(570, 85, 60, 30, "до:");

        inp_val_min.set_value("0.0"); inp_val_max.set_value("100.0");
        inp_t_min.set_value("273.15"); inp_t_max.set_value("1000.0");
        inp_val_min.deactivate(); inp_val_max.deactivate();
        inp_t_min.deactivate(); inp_t_max.deactivate();

        let mut btn_apply_limits = Button::new(650, 85, 150, 30, "Применить масштаб");
        btn_apply_limits.deactivate();

        let mut plot_frame = Frame::new(20, 130, 1000, 515, "");
        plot_frame.set_color(Color::White);
        plot_frame.set_frame(FrameType::FlatBox);

        group.end();

        // --- Коллбеки ---
        choice_plot_type.set_callback({
            let s = sender.clone();
            let mut ivm = inp_val_min.clone();
            move |c| {
                let pt = match c.value() {
                    0 => { ivm.set_label("p, МПа от:"); PlotType::PT },
                    1 => { ivm.set_label("rho, кг/м3 от:"); PlotType::RhoT },
                    _ => { ivm.set_label("v, м3/кг от:"); PlotType::VT },
                };
                info!("Смена типа графика на {:?}", pt);
                s.send(Message::ChangePlotType(pt));
            }
        });

        check_dome.set_callback({ let s = sender.clone(); move |c| s.send(Message::ToggleDome(c.value())) });
        check_swap.set_callback({ let s = sender.clone(); move |c| s.send(Message::ToggleSwapAxes(c.value())) });

        check_autoscale.set_callback({
            let s = sender.clone();
            let mut iv_min = inp_val_min.clone(); let mut iv_max = inp_val_max.clone();
            let mut it_min = inp_t_min.clone(); let mut it_max = inp_t_max.clone();
            let mut btn_apply = btn_apply_limits.clone();
            move |c| {
                let auto = c.value();
                debug!("Автомасштаб установлен в {}", auto);
                if auto {
                    iv_min.deactivate(); iv_max.deactivate(); it_min.deactivate(); it_max.deactivate(); btn_apply.deactivate();
                } else {
                    iv_min.activate(); iv_max.activate(); it_min.activate(); it_max.activate(); btn_apply.activate();
                }
                s.send(Message::SetAutoscale(auto));
            }
        });

        btn_apply_limits.set_callback({
            let s = sender.clone();
            let ivm = inp_val_min.clone(); let ivx = inp_val_max.clone();
            let itm = inp_t_min.clone(); let itx = inp_t_max.clone();
            move |_| {
                let vmin = ivm.value().replace(',', ".").parse().unwrap_or(0.0);
                let vmax = ivx.value().replace(',', ".").parse().unwrap_or(100.0);
                let tmin = itm.value().replace(',', ".").parse().unwrap_or(273.15);
                let tmax = itx.value().replace(',', ".").parse().unwrap_or(1000.0);
                info!("Применение ручного масштаба: [{}-{}] и [{}-{}]", vmin, vmax, tmin, tmax);
                s.send(Message::ApplyPlotLimits(vmin, vmax, tmin, tmax));
            }
        });

        btn_select_data.set_callback({ let s = sender.clone(); move |_| s.send(Message::SelectData) });
        btn_export_plot.set_callback({ let s = sender.clone(); move |_| s.send(Message::ExportPlot) });

        Self { group, choice_plot_type, plot_frame, inp_val_min, inp_val_max, inp_t_min, inp_t_max, btn_apply_limits }
    }

    pub fn redraw_plot(&mut self, state: &AppState) {
        let fw = self.plot_frame.w();
        let fh = self.plot_frame.h();
        if fw <= 0 || fh <= 0 { return; }

        debug!("Перерисовка графика: {}x{}", fw, fh);
        let buffer = render_plot_to_buffer(state, fw as u32, fh as u32);
        if let Ok(img) = RgbImage::new(&buffer, fw, fh, ColorDepth::Rgb8) {
            self.plot_frame.set_image(Some(img));
            self.plot_frame.redraw();
        }
    }
}