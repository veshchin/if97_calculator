use crate::plot::renderer::render_plot_to_buffer;
use crate::state::{AppState, Message};
use fltk::app::Sender;
use fltk::{
    button::*,
    enums::*,
    frame::*,
    group::*,
    image::RgbImage,
    input::*,
    menu::*,
    prelude::*,
};
use if97_app_api::DiagramKind;

fn choice_index_to_plot(index: i32) -> DiagramKind {
    match index {
        1 => DiagramKind::Pv,
        2 => DiagramKind::Ps,
        3 => DiagramKind::Ph,
        4 => DiagramKind::Tv,
        5 => DiagramKind::Ts,
        6 => DiagramKind::Th,
        7 => DiagramKind::Hs,
        _ => DiagramKind::Pt,
    }
}

fn plot_to_choice_index(kind: DiagramKind) -> i32 {
    match kind {
        DiagramKind::Pt => 0,
        DiagramKind::Pv => 1,
        DiagramKind::Ps => 2,
        DiagramKind::Ph => 3,
        DiagramKind::Tv => 4,
        DiagramKind::Ts => 5,
        DiagramKind::Th => 6,
        DiagramKind::Hs => 7,
    }
}

fn axis_labels(kind: DiagramKind, swap_axes: bool) -> (&'static str, &'static str) {
    let (x, y) = match kind {
        DiagramKind::Pt => ("X: Температура T", "Y: Давление p"),
        DiagramKind::Pv => ("X: Удельный объем v", "Y: Давление p"),
        DiagramKind::Ps => ("X: Энтропия s", "Y: Давление p"),
        DiagramKind::Ph => ("X: Энтальпия h", "Y: Давление p"),
        DiagramKind::Tv => ("X: Удельный объем v", "Y: Температура T"),
        DiagramKind::Ts => ("X: Энтропия s", "Y: Температура T"),
        DiagramKind::Th => ("X: Энтальпия h", "Y: Температура T"),
        DiagramKind::Hs => ("X: Энтропия s", "Y: Энтальпия h"),
    };
    if swap_axes { (y, x) } else { (x, y) }
}

#[allow(dead_code)]
pub struct PlotTab {
    pub group: Group,
    pub choice_plot_type: Choice,
    pub plot_frame: Frame,
    pub inp_x_min: Input,
    pub inp_x_max: Input,
    pub inp_y_min: Input,
    pub inp_y_max: Input,
    pub axis_x_label: Frame,
    pub axis_y_label: Frame,
}

impl PlotTab {
    pub fn new(sender: Sender<Message>) -> Self {
        let group = Group::new(10, 35, 1030, 655, " Графики ");

        let mut choice_plot_type = Choice::new(20, 45, 180, 30, "Тип диаграммы:");
        choice_plot_type.set_align(Align::TopLeft);
        choice_plot_type.add_choice("p-T|p-v|p-s|p-h|T-v|T-s|T-h|h-s");
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

        let mut axis_x_label = Frame::new(155, 86, 110, 24, "X: Температура T");
        axis_x_label.set_align(Align::Left | Align::Inside);
        let mut inp_x_min = Input::new(270, 85, 70, 30, "");
        let mut inp_x_max = Input::new(350, 85, 70, 30, "");

        let mut axis_y_label = Frame::new(435, 86, 110, 24, "Y: Давление p");
        axis_y_label.set_align(Align::Left | Align::Inside);
        let mut inp_y_min = Input::new(550, 85, 70, 30, "");
        let mut inp_y_max = Input::new(630, 85, 70, 30, "");

        inp_x_min.set_value("273.15");
        inp_x_max.set_value("1000.0");
        inp_y_min.set_value("0.001");
        inp_y_max.set_value("100.0");
        inp_x_min.deactivate();
        inp_x_max.deactivate();
        inp_y_min.deactivate();
        inp_y_max.deactivate();

        let mut btn_apply_limits = Button::new(720, 85, 150, 30, "Применить масштаб");
        btn_apply_limits.deactivate();

        let mut plot_frame = Frame::new(20, 130, 1000, 515, "");
        plot_frame.set_color(Color::White);
        plot_frame.set_frame(FrameType::FlatBox);

        group.end();

        choice_plot_type.set_callback({
            let s = sender.clone();
            move |c| s.send(Message::ChangePlotType(choice_index_to_plot(c.value())))
        });

        check_dome.set_callback({
            let s = sender.clone();
            move |c| s.send(Message::ToggleDome(c.value()))
        });
        check_swap.set_callback({
            let s = sender.clone();
            move |c| s.send(Message::ToggleSwapAxes(c.value()))
        });

        check_autoscale.set_callback({
            let s = sender.clone();
            let mut x_min = inp_x_min.clone();
            let mut x_max = inp_x_max.clone();
            let mut y_min = inp_y_min.clone();
            let mut y_max = inp_y_max.clone();
            let mut btn_apply = btn_apply_limits.clone();
            move |c| {
                let auto = c.value();
                if auto {
                    x_min.deactivate();
                    x_max.deactivate();
                    y_min.deactivate();
                    y_max.deactivate();
                    btn_apply.deactivate();
                } else {
                    x_min.activate();
                    x_max.activate();
                    y_min.activate();
                    y_max.activate();
                    btn_apply.activate();
                }
                s.send(Message::SetAutoscale(auto));
            }
        });

        btn_apply_limits.set_callback({
            let s = sender.clone();
            let x_min = inp_x_min.clone();
            let x_max = inp_x_max.clone();
            let y_min = inp_y_min.clone();
            let y_max = inp_y_max.clone();
            move |_| {
                s.send(Message::ApplyPlotLimits(
                    x_min.value().replace(',', ".").parse().unwrap_or(0.0),
                    x_max.value().replace(',', ".").parse().unwrap_or(1.0),
                    y_min.value().replace(',', ".").parse().unwrap_or(0.0),
                    y_max.value().replace(',', ".").parse().unwrap_or(1.0),
                ));
            }
        });

        btn_select_data.set_callback({
            let s = sender.clone();
            move |_| s.send(Message::SelectData)
        });

        btn_export_plot.set_callback({
            let s = sender.clone();
            move |_| s.send(Message::ExportPlot)
        });

        Self {
            group,
            choice_plot_type,
            plot_frame,
            inp_x_min,
            inp_x_max,
            inp_y_min,
            inp_y_max,
            axis_x_label,
            axis_y_label,
        }
    }

    pub fn sync_controls(&mut self, state: &AppState) {
        self.choice_plot_type
            .set_value(plot_to_choice_index(state.plot_type));
        let (x_label, y_label) = axis_labels(state.plot_type, state.swap_axes);
        self.axis_x_label.set_label(x_label);
        self.axis_y_label.set_label(y_label);
        self.inp_x_min.set_value(&format!("{:.6}", state.custom_limits.0));
        self.inp_x_max.set_value(&format!("{:.6}", state.custom_limits.1));
        self.inp_y_min.set_value(&format!("{:.6}", state.custom_limits.2));
        self.inp_y_max.set_value(&format!("{:.6}", state.custom_limits.3));
    }

    pub fn redraw_plot(&mut self, state: &AppState) {
        self.sync_controls(state);

        let fw = self.plot_frame.w();
        let fh = self.plot_frame.h();
        if fw <= 0 || fh <= 0 {
            return;
        }

        let buffer = render_plot_to_buffer(state, fw as u32, fh as u32);
        if let Ok(img) = RgbImage::new(&buffer, fw, fh, ColorDepth::Rgb8) {
            self.plot_frame.set_image(Some(img));
            self.plot_frame.redraw();
        }
    }
}
