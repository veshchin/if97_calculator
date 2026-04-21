//! UI-вкладка построения графиков.

use crate::plot::renderer::render_plot_to_buffer;
use crate::state::{AppState, Message};
use fltk::app::Sender;
use fltk::{
    button::*, enums::*, frame::*, group::*, image::RgbImage, input::*, menu::Choice, prelude::*,
};
use if97_app_api::AxisVar;

fn axis_label_ru(var: AxisVar) -> &'static str {
    match var {
        AxisVar::P => "Давление p, МПа",
        AxisVar::T => "Температура T, К",
        AxisVar::V => "Удельный объем v, м3/кг",
        AxisVar::Rho => "Плотность rho, кг/м3",
        AxisVar::H => "Энтальпия h, кДж/кг",
        AxisVar::S => "Энтропия s, кДж/(кг·К)",
        AxisVar::U => "Внутренняя энергия u, кДж/кг",
        AxisVar::Cp => "Изобарная теплоемкость cp, кДж/(кг·К)",
        AxisVar::W => "Скорость звука w, м/с",
        AxisVar::X => "Степень сухости x",
    }
}

#[allow(dead_code)]
/// Вкладка UI для отрисовки графиков и экспорта в PNG.
pub struct PlotTab {
    /// Корневой контейнер вкладки.
    pub group: Group,
    /// Выпадающий список выбора величины по оси X.
    pub choice_x: Choice,
    /// Выпадающий список выбора величины по оси Y.
    pub choice_y: Choice,
    /// Логарифмическая шкала по X.
    pub check_x_log: CheckButton,
    /// Логарифмическая шкала по Y.
    pub check_y_log: CheckButton,
    /// Область отрисовки (изображение устанавливается как `RgbImage`).
    pub plot_frame: Frame,
    /// Ручной минимум X.
    pub inp_x_min: Input,
    /// Ручной максимум X.
    pub inp_x_max: Input,
    /// Ручной минимум Y.
    pub inp_y_min: Input,
    /// Ручной максимум Y.
    pub inp_y_max: Input,
    /// Подпись оси X.
    pub axis_x_label: Frame,
    /// Подпись оси Y.
    pub axis_y_label: Frame,
}

impl PlotTab {
    /// Создает вкладку и настраивает callback'и для отправки [`Message`] в обработчик.
    pub fn new(sender: Sender<Message>) -> Self {
        let group = Group::new(10, 35, 1030, 655, " Графики ");

        let axis_choices = AxisVar::ALL
            .iter()
            .map(|&var| axis_label_ru(var))
            .collect::<Vec<_>>()
            .join("|");

        let mut choice_x = Choice::new(20, 45, 170, 30, "X");
        choice_x.add_choice(&axis_choices);
        choice_x.set_value(AxisVar::T.to_index() as i32);

        let mut choice_y = Choice::new(210, 45, 170, 30, "Y");
        choice_y.add_choice(&axis_choices);
        choice_y.set_value(AxisVar::P.to_index() as i32);

        let mut check_dome = CheckButton::new(400, 45, 110, 30, "Купол");
        check_dome.set_value(true);

        let mut check_x_log = CheckButton::new(520, 45, 80, 30, "log X");
        check_x_log.set_value(false);
        let mut check_y_log = CheckButton::new(610, 45, 80, 30, "log Y");
        check_y_log.set_value(false);

        let mut check_autoscale = CheckButton::new(20, 85, 120, 30, "Автомасштаб");
        check_autoscale.set_value(true);

        let mut axis_x_label = Frame::new(155, 86, 260, 24, "X: Температура T, К");
        axis_x_label.set_align(Align::Left | Align::Inside);
        let mut inp_x_min = Input::new(420, 85, 70, 30, "");
        let mut inp_x_max = Input::new(500, 85, 70, 30, "");

        let mut axis_y_label = Frame::new(585, 86, 260, 24, "Y: Давление p, МПа");
        axis_y_label.set_align(Align::Left | Align::Inside);
        let mut inp_y_min = Input::new(850, 85, 70, 30, "");
        let mut inp_y_max = Input::new(930, 85, 70, 30, "");

        inp_x_min.set_value("273.15");
        inp_x_max.set_value("1000.0");
        inp_y_min.set_value("0.001");
        inp_y_max.set_value("100.0");
        inp_x_min.deactivate();
        inp_x_max.deactivate();
        inp_y_min.deactivate();
        inp_y_max.deactivate();

        let mut btn_apply_limits = Button::new(20, 125, 150, 30, "Применить масштаб");
        btn_apply_limits.deactivate();

        let mut btn_select_data = Button::new(180, 125, 160, 30, "Выбрать данные");

        let mut btn_export_plot = Button::new(350, 125, 160, 30, "Экспорт PNG");
        btn_export_plot.set_color(Color::from_rgb(100, 149, 237));
        btn_export_plot.set_label_color(Color::White);

        let mut btn_export_plot_svg = Button::new(520, 125, 160, 30, "Экспорт SVG");
        btn_export_plot_svg.set_color(Color::from_rgb(46, 139, 87));
        btn_export_plot_svg.set_label_color(Color::White);

        let mut plot_frame = Frame::new(20, 165, 1000, 480, "");
        plot_frame.set_color(Color::White);
        plot_frame.set_frame(FrameType::FlatBox);

        group.end();

        choice_x.set_callback({
            let s = sender.clone();
            move |c| {
                let idx = c.value().max(0) as usize;
                let var = AxisVar::from_index(idx);
                c.set_value(var.to_index() as i32);
                s.send(Message::SetPlotX(var));
            }
        });
        choice_y.set_callback({
            let s = sender.clone();
            move |c| {
                let idx = c.value().max(0) as usize;
                let var = AxisVar::from_index(idx);
                c.set_value(var.to_index() as i32);
                s.send(Message::SetPlotY(var));
            }
        });

        check_dome.set_callback({
            let s = sender.clone();
            move |c| s.send(Message::ToggleDome(c.value()))
        });
        check_x_log.set_callback({
            let s = sender.clone();
            move |c| s.send(Message::TogglePlotXLog(c.value()))
        });
        check_y_log.set_callback({
            let s = sender.clone();
            move |c| s.send(Message::TogglePlotYLog(c.value()))
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
        btn_export_plot_svg.set_callback({
            let s = sender.clone();
            move |_| s.send(Message::ExportPlotSvg)
        });

        Self {
            group,
            choice_x,
            choice_y,
            check_x_log,
            check_y_log,
            plot_frame,
            inp_x_min,
            inp_x_max,
            inp_y_min,
            inp_y_max,
            axis_x_label,
            axis_y_label,
        }
    }

    /// Синхронизирует контролы вкладки с текущим состоянием приложения.
    pub fn sync_controls(&mut self, state: &AppState) {
        self.choice_x.set_value(state.plot_x.to_index() as i32);
        self.choice_y.set_value(state.plot_y.to_index() as i32);
        self.check_x_log.set_value(state.plot_x_log);
        self.check_y_log.set_value(state.plot_y_log);

        self.axis_x_label
            .set_label(&format!("X: {}", axis_label_ru(state.plot_x)));
        self.axis_y_label
            .set_label(&format!("Y: {}", axis_label_ru(state.plot_y)));

        self.inp_x_min
            .set_value(&format!("{:.6}", state.custom_limits.0));
        self.inp_x_max
            .set_value(&format!("{:.6}", state.custom_limits.1));
        self.inp_y_min
            .set_value(&format!("{:.6}", state.custom_limits.2));
        self.inp_y_max
            .set_value(&format!("{:.6}", state.custom_limits.3));
    }

    /// Перерисовывает график в `plot_frame` на основании `state`.
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
