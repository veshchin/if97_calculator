use crate::state::Message;
use fltk::app;
use fltk::app::Sender;
use fltk::{
    browser::HoldBrowser,
    button::Button,
    enums::{Align, Color, Font, FrameType},
    frame::Frame,
    group::{Flex, Group},
    input::Input,
    menu::Choice,
    prelude::*,
};
use if97_app_api::InputMode;

fn mode_to_choice_index(mode: InputMode) -> i32 {
    match mode {
        InputMode::Pt => 0,
        InputMode::Rhot => 1,
        InputMode::Ph => 2,
        InputMode::Ps => 3,
        InputMode::Px => 4,
    }
}

fn choice_index_to_mode(index: i32) -> InputMode {
    match index {
        1 => InputMode::Rhot,
        2 => InputMode::Ph,
        3 => InputMode::Ps,
        4 => InputMode::Px,
        _ => InputMode::Pt,
    }
}

fn mode_labels(mode: InputMode) -> (&'static str, &'static str) {
    match mode {
        InputMode::Pt => ("Давление p, МПа", "Температура T, К"),
        InputMode::Rhot => ("Плотность rho, кг/м3", "Температура T, К"),
        InputMode::Ph => ("Давление p, МПа", "Энтальпия h, кДж/кг"),
        InputMode::Ps => ("Давление p, МПа", "Энтропия s, кДж/(кг·К)"),
        InputMode::Px => ("Давление p, МПа", "Степень сухости x"),
    }
}

#[allow(dead_code)]
pub struct SingleTab {
    pub group: Group,
    pub choice_mode: Choice,
    pub choice_precision: Choice,
    pub input_a: Input,
    pub input_b: Input,
    pub save_name: Input,
    pub save_button: Button,
    pub load_button: Button,
    pub delete_button: Button,
    pub saved_browser: HoldBrowser,
    pub res_frame: Frame,
}

impl SingleTab {
    pub fn new(sender: Sender<Message>) -> Self {
        let group = Group::new(10, 35, 1030, 655, " Одиночный расчет ");

        let mut root = Flex::new(20, 45, 1000, 590, "").row();
        root.set_pad(18);

        let mut left = Flex::default().column();
        left.set_pad(14);

        let mut title = Frame::default().with_size(0, 34).with_label("Одиночный расчет");
        title.set_label_size(22);
        title.set_label_font(Font::HelveticaBold);
        title.set_align(Align::Left | Align::Inside);

        let mut choice_row = Flex::default().with_size(0, 70).row();
        choice_row.set_pad(12);

        let mut choice_mode = Choice::default().with_label("Режим расчета");
        choice_mode.set_align(Align::TopLeft);
        choice_mode.add_choice("p-T|rho-T|p-h|p-s|p-x");
        choice_mode.set_value(0);

        let mut choice_precision = Choice::default().with_label("Точность");
        choice_precision.set_align(Align::TopLeft);
        choice_precision.add_choice("0|1|2|3|4|5|6|7|8|9|10");
        choice_precision.set_value(4);

        choice_row.fixed(&choice_precision, 120);
        choice_row.end();

        let mut input_row = Flex::default().with_size(0, 70).row();
        input_row.set_pad(12);

        let mut input_a = Input::default().with_label("Давление p, МПа");
        input_a.set_align(Align::TopLeft);
        let mut input_b = Input::default().with_label("Температура T, К");
        input_b.set_align(Align::TopLeft);

        input_row.end();

        let mut save_row = Flex::default().with_size(0, 42).row();
        save_row.set_pad(10);

        let mut save_name = Input::default().with_label("");
        save_name.set_tooltip("Имя сохраненной точки");
        save_name.set_value("Точка 1");

        let mut save_button = Button::default().with_label("Сохранить / обновить");
        save_button.set_color(Color::from_rgb(25, 135, 84));
        save_button.set_label_color(Color::White);

        save_row.fixed(&save_button, 180);
        save_row.end();

        let mut res_frame = Frame::default().with_label("Ожидание данных...");
        res_frame.set_frame(FrameType::DownBox);
        res_frame.set_color(Color::White);
        res_frame.set_align(Align::Left | Align::Inside | Align::Top);
        res_frame.set_label_size(14);

        left.end();

        let mut right = Flex::default().column();
        right.set_pad(10);

        let mut saved_title = Frame::default()
            .with_size(0, 28)
            .with_label("Сохраненные точки");
        saved_title.set_label_font(Font::HelveticaBold);
        saved_title.set_align(Align::Left | Align::Inside);

        let mut saved_browser = HoldBrowser::default();
        saved_browser.set_text_size(13);

        let mut saved_actions = Flex::default().with_size(0, 38).row();
        saved_actions.set_pad(10);

        let mut load_button = Button::default().with_label("Загрузить");
        let mut delete_button = Button::default().with_label("Удалить");
        delete_button.set_color(Color::from_rgb(220, 53, 69));
        delete_button.set_label_color(Color::White);

        saved_actions.fixed(&load_button, 120);
        saved_actions.fixed(&delete_button, 120);
        saved_actions.end();

        right.end();

        root.fixed(&right, 260);
        root.end();
        group.end();

        choice_mode.set_callback({
            let mut ia = input_a.clone();
            let mut ib = input_b.clone();
            let i_a = input_a.clone();
            let i_b = input_b.clone();
            let cm = choice_mode.clone();
            let s = sender.clone();
            move |c| {
                let mode = choice_index_to_mode(c.value());
                let (label_a, label_b) = mode_labels(mode);
                ia.set_label(label_a);
                ib.set_label(label_b);
                s.send(Message::CalculateSingle {
                    mode,
                    raw_a: i_a.value(),
                    raw_b: i_b.value(),
                });
                let _ = cm.value();
            }
        });

        input_a.set_trigger(fltk::enums::CallbackTrigger::Changed);
        input_a.set_callback({
            let s = sender.clone();
            let i_a = input_a.clone();
            let i_b = input_b.clone();
            let cm = choice_mode.clone();
            move |_| {
                s.send(Message::CalculateSingle {
                    mode: choice_index_to_mode(cm.value()),
                    raw_a: i_a.value(),
                    raw_b: i_b.value(),
                });
            }
        });

        input_b.set_trigger(fltk::enums::CallbackTrigger::Changed);
        input_b.set_callback({
            let s = sender.clone();
            let i_a = input_a.clone();
            let i_b = input_b.clone();
            let cm = choice_mode.clone();
            move |_| {
                s.send(Message::CalculateSingle {
                    mode: choice_index_to_mode(cm.value()),
                    raw_a: i_a.value(),
                    raw_b: i_b.value(),
                });
            }
        });

        choice_precision.set_callback({
            let s = sender.clone();
            move |c| {
                let precision = c.value().max(0) as usize;
                s.send(Message::SetSinglePrecision(precision));
            }
        });

        save_button.set_callback({
            let s = sender.clone();
            let i_a = input_a.clone();
            let i_b = input_b.clone();
            let cm = choice_mode.clone();
            let save_name = save_name.clone();
            move |_| {
                s.send(Message::SaveSinglePoint {
                    name: save_name.value(),
                    mode: choice_index_to_mode(cm.value()),
                    raw_a: i_a.value(),
                    raw_b: i_b.value(),
                });
            }
        });

        load_button.set_callback({
            let s = sender.clone();
            let browser = saved_browser.clone();
            move |_| {
                let index = browser.value();
                if index > 0 {
                    s.send(Message::LoadSavedPoint((index - 1) as usize));
                }
            }
        });

        delete_button.set_callback({
            let s = sender.clone();
            let browser = saved_browser.clone();
            move |_| {
                let index = browser.value();
                if index > 0 {
                    s.send(Message::DeleteSavedPoint((index - 1) as usize));
                }
            }
        });

        saved_browser.set_callback({
            let s = sender.clone();
            move |browser| {
                let index = browser.value();
                if app::event_clicks() {
                    if index > 0 {
                        s.send(Message::LoadSavedPoint((index - 1) as usize));
                    }
                }
            }
        });

        Self {
            group,
            choice_mode,
            choice_precision,
            input_a,
            input_b,
            save_name,
            save_button,
            load_button,
            delete_button,
            saved_browser,
            res_frame,
        }
    }

    pub fn set_mode(&mut self, mode: InputMode) {
        self.choice_mode.set_value(mode_to_choice_index(mode));
        let (label_a, label_b) = mode_labels(mode);
        self.input_a.set_label(label_a);
        self.input_b.set_label(label_b);
    }

    pub fn set_precision(&mut self, precision: usize) {
        self.choice_precision.set_value(precision.min(10) as i32);
    }

    pub fn set_request(&mut self, mode: InputMode, raw_a: &str, raw_b: &str) {
        self.set_mode(mode);
        self.input_a.set_value(raw_a);
        self.input_b.set_value(raw_b);
    }

    pub fn set_save_name(&mut self, value: &str) {
        self.save_name.set_value(value);
    }

    pub fn sync_saved_points(&mut self, labels: &[String]) {
        self.saved_browser.clear();
        for label in labels {
            self.saved_browser.add(label);
        }
    }

    pub fn update_result(&mut self, text: &str, is_error: bool) {
        self.res_frame.set_label(text);
        self.res_frame
            .set_label_color(if is_error { Color::Red } else { Color::Black });
    }
}
