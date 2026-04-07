// File: src/ui/single.rs

use crate::state::Message;
use fltk::app::Sender;
use fltk::{button::*, enums::*, frame::*, group::*, input::*, menu::*, prelude::*};
use tracing::{debug, info};

pub struct SingleTab {
    pub group: Group,
    pub choice_mode: Choice,
    pub input_a: Input,
    pub input_b: Input,
    pub res_frame: Frame,
}

impl SingleTab {
    pub fn new(sender: Sender<Message>) -> Self {
        let group = Group::new(10, 35, 1030, 655, " Одиночный расчет ");

        let mut instruction_single = Frame::new(
            20,
            45,
            1010,
            40,
            "Инструкция: Выберите режим расчета и введите параметры. \nВнимание: Дробная часть числа должна отделяться ТОЧКОЙ (например, 14.5, а не 14,5).",
        );
        instruction_single.set_align(Align::Left | Align::Inside);
        instruction_single.set_label_color(Color::Dark3);
        instruction_single.set_label_font(Font::HelveticaItalic);

        let mut flex_single = Flex::new(20, 90, 400, 300, "").column();
        flex_single.set_pad(15);

        let mut choice_mode = Choice::default().with_label("Что известно:");
        choice_mode.set_align(Align::TopLeft);
        choice_mode.add_choice("Давление и Температура (p, T)|Плотность и Температура (rho, T)|Давление и Энтальпия (p, h)|Давление и Энтропия (p, s)|Линия насыщения (p, x)");
        choice_mode.set_value(0);

        let mut input_a = Input::default().with_label("Давление (p), МПа:");
        input_a.set_align(Align::TopLeft);
        let mut input_b = Input::default().with_label("Температура (T), К:");
        input_b.set_align(Align::TopLeft);

        let mut btn_row_single = Flex::default().row();
        let mut btn_calc = Button::default().with_label("Рассчитать");
        btn_calc.set_color(Color::from_rgb(0, 120, 215));
        btn_calc.set_label_color(Color::White);

        let mut btn_save_single = Button::default().with_label("Сохранить точку");
        btn_save_single.set_color(Color::from_rgb(34, 139, 34));
        btn_save_single.set_label_color(Color::White);
        btn_row_single.end();

        let mut res_frame = Frame::default().with_label("Ожидание данных...");
        res_frame.set_frame(FrameType::DownBox);
        res_frame.set_color(Color::White);

        flex_single.end();
        group.end();

        // --- Коллбеки ---
        choice_mode.set_callback({
            let mut ia = input_a.clone();
            let mut ib = input_b.clone();
            move |c| {
                let val = c.value();
                debug!("Смена режима одиночного расчета на индекс {}", val);
                match val {
                    0 => {
                        ia.set_label("Давление (p), МПа:");
                        ib.set_label("Температура (T), К:");
                    }
                    1 => {
                        ia.set_label("Плотность (rho), кг/м3:");
                        ib.set_label("Температура (T), К:");
                    }
                    2 => {
                        ia.set_label("Давление (p), МПа:");
                        ib.set_label("Энтальпия (h), кДж/кг:");
                    }
                    3 => {
                        ia.set_label("Давление (p), МПа:");
                        ib.set_label("Энтропия (s), кДж/(кг*К):");
                    }
                    4 => {
                        ia.set_label("Давление (p), МПа:");
                        ib.set_label("Степень сухости (x), 0.0-1.0:");
                    }
                    _ => {}
                }
                ia.redraw();
                ib.redraw();
            }
        });

        btn_calc.set_callback({
            let s = sender.clone();
            let i_a = input_a.clone();
            let i_b = input_b.clone();
            let cm = choice_mode.clone();
            move |_| {
                let val_a = i_a
                    .value()
                    .replace(',', ".")
                    .parse::<f64>()
                    .unwrap_or(f64::NAN);
                let val_b = i_b
                    .value()
                    .replace(',', ".")
                    .parse::<f64>()
                    .unwrap_or(f64::NAN);
                debug!(
                    "Клик 'Рассчитать': mode={}, a={}, b={}",
                    cm.value(),
                    val_a,
                    val_b
                );
                s.send(Message::CalculateSingle {
                    mode: cm.value(),
                    val_a,
                    val_b,
                });
            }
        });

        btn_save_single.set_callback({
            let s = sender.clone();
            move |_| {
                info!("Клик 'Сохранить точку'");
                s.send(Message::SaveSinglePoint);
            }
        });

        Self {
            group,
            choice_mode,
            input_a,
            input_b,
            res_frame,
        }
    }

    pub fn update_result(&mut self, text: &str, is_error: bool) {
        self.res_frame.set_label(text);
        self.res_frame
            .set_label_color(if is_error { Color::Red } else { Color::Black });
    }
}
