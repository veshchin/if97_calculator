//! UI-вкладка табличного расчета.

use crate::state::{BatchRow, Message};
use fltk::app;
use fltk::app::Sender;
use fltk::browser::HoldBrowser;
use fltk::button::Button;
use fltk::draw;
use fltk::enums::{Align, CallbackTrigger, Color, Font, FrameType};
use fltk::frame::Frame;
use fltk::group::{Flex, Group};
use fltk::input::{Input, MultilineInput};
use fltk::menu::Choice;
use fltk::prelude::*;
use fltk::table::{Table, TableContext};
use if97_app_api::InputMode;
use std::cell::RefCell;
use std::rc::Rc;

const TABLE_HEADERS: [&str; 10] = [
    "p (МПа)",
    "T (K)",
    "v (м3/кг)",
    "rho (кг/м3)",
    "h (кДж/кг)",
    "s (кДж/кгК)",
    "u (кДж/кг)",
    "cp (кДж/кгК)",
    "w (м/с)",
    "Регион",
];

fn choice_index_to_mode(index: i32) -> InputMode {
    match index {
        1 => InputMode::Rhot,
        2 => InputMode::Ph,
        3 => InputMode::Ps,
        4 => InputMode::Px,
        _ => InputMode::Pt,
    }
}

fn mode_to_choice_index(mode: InputMode) -> i32 {
    match mode {
        InputMode::Pt => 0,
        InputMode::Rhot => 1,
        InputMode::Ph => 2,
        InputMode::Ps => 3,
        InputMode::Px => 4,
    }
}

fn mode_labels(mode: InputMode) -> (&'static str, &'static str) {
    match mode {
        InputMode::Pt => ("p, МПа", "T, К"),
        InputMode::Rhot => ("rho, кг/м3", "T, К"),
        InputMode::Ph => ("p, МПа", "h, кДж/кг"),
        InputMode::Ps => ("p, МПа", "s, кДж/кгК"),
        InputMode::Px => ("p, МПа", "x"),
    }
}

fn format_value(value: f64, precision: usize) -> String {
    if value.is_infinite() {
        if value.is_sign_negative() {
            "-∞".to_string()
        } else {
            "∞".to_string()
        }
    } else if value.is_nan() {
        "NaN".to_string()
    } else {
        format!("{:.*}", precision, value)
    }
}

#[derive(Default)]
struct TableDisplayModel {
    rows: Vec<DisplayRow>,
    selected_row: Option<usize>,
    selected_col: Option<usize>,
}

#[derive(Clone)]
struct DisplayRow {
    line_no: usize,
    values: Vec<String>,
    error: Option<String>,
}

fn draw_header(text: &str, x: i32, y: i32, w: i32, h: i32, highlighted: bool) {
    draw::push_clip(x, y, w, h);
    let color = if highlighted {
        Color::from_rgb(204, 229, 255)
    } else {
        Color::FrameDefault
    };
    draw::draw_box(FrameType::ThinUpBox, x, y, w, h, color);
    draw::set_draw_color(Color::Black);
    draw::set_font(Font::HelveticaBold, 13);
    draw::draw_text2(text, x + 4, y, w - 8, h, Align::Center);
    draw::pop_clip();
}

fn draw_table_cell(text: &str, x: i32, y: i32, w: i32, h: i32, bg: Color, text_color: Color) {
    draw::push_clip(x, y, w, h);
    draw::set_draw_color(bg);
    draw::draw_rectf(x, y, w, h);
    draw::set_draw_color(text_color);
    draw::set_font(Font::Helvetica, 13);
    draw::draw_text2(text, x + 6, y, w - 12, h, Align::Inside | Align::Left);
    draw::set_draw_color(Color::from_rgb(222, 226, 230));
    draw::draw_rect(x, y, w, h);
    draw::pop_clip();
}

#[allow(dead_code)]
/// Вкладка UI для табличного расчета (ввод текстом, пересчет и экспорт результата).
pub struct BatchTab {
    /// Корневой контейнер вкладки.
    pub group: Group,
    /// Выбор режима расчета.
    pub choice_mode: Choice,
    /// Выбор точности вывода.
    pub choice_precision: Choice,
    /// Поле ввода табличных данных.
    pub input_area: MultilineInput,
    /// Поле ввода имени для сохранения таблицы.
    pub save_name: Input,
    /// Список сохраненных таблиц.
    pub saved_browser: HoldBrowser,
    /// Генератор: начало диапазона первого параметра.
    pub gen_v1_from: Input,
    /// Генератор: конец диапазона первого параметра.
    pub gen_v1_to: Input,
    /// Генератор: шаг первого параметра.
    pub gen_v1_step: Input,
    /// Генератор: начало диапазона второго параметра.
    pub gen_v2_from: Input,
    /// Генератор: конец диапазона второго параметра.
    pub gen_v2_to: Input,
    /// Генератор: шаг второго параметра.
    pub gen_v2_step: Input,
    /// Подпись для первого параметра генератора.
    pub gen_v1_label: Frame,
    /// Подпись для второго параметра генератора.
    pub gen_v2_label: Frame,
    /// Таблица результата расчета.
    pub result_table: Table,
    model: Rc<RefCell<TableDisplayModel>>,
}

impl BatchTab {
    /// Создает вкладку и настраивает callback'и для отправки [`Message`] в обработчик.
    pub fn new(sender: Sender<Message>) -> Self {
        let group = Group::new(10, 35, 1030, 655, " Табличный расчет ");

        let mut root = Flex::new(20, 45, 1000, 590, "").column();
        root.set_pad(12);

        let mut top = Flex::default().with_size(0, 40).row();
        top.set_pad(10);

        let mut choice_mode = Choice::default().with_label("");
        choice_mode.add_choice("p-T|rho-T|p-h|p-s|p-x");
        choice_mode.set_value(0);

        let mut choice_precision = Choice::default().with_label("");
        choice_precision.add_choice("0|1|2|3|4|5|6|7|8|9|10");
        choice_precision.set_value(4);

        let mut btn_open = Button::default().with_label("Открыть");
        let mut btn_export = Button::default().with_label("Экспорт CSV");
        let mut btn_clear = Button::default().with_label("Сброс выбора");
        let save_name = Input::default();
        let mut btn_save = Button::default().with_label("Сохранить / обновить");
        btn_save.set_color(Color::from_rgb(25, 135, 84));
        btn_save.set_label_color(Color::White);

        top.fixed(&choice_mode, 120);
        top.fixed(&choice_precision, 80);
        top.fixed(&btn_open, 110);
        top.fixed(&btn_export, 120);
        top.fixed(&btn_clear, 120);
        top.fixed(&btn_save, 180);
        top.end();

        let mut generator = Flex::default().with_size(0, 44).row();
        generator.set_pad(8);

        let mut gen_v1_label = Frame::default().with_label("p, МПа");
        gen_v1_label.set_align(Align::Right | Align::Inside);
        let gen_v1_from = Input::default();
        let gen_v1_to = Input::default();
        let gen_v1_step = Input::default();
        let mut gen_sep = Frame::default().with_label("");
        gen_sep.set_frame(FrameType::FlatBox);

        let mut gen_v2_label = Frame::default().with_label("T, К");
        gen_v2_label.set_align(Align::Right | Align::Inside);
        let gen_v2_from = Input::default();
        let gen_v2_to = Input::default();
        let gen_v2_step = Input::default();
        let mut btn_generate = Button::default().with_label("Добавить строки");
        btn_generate.set_color(Color::from_rgb(13, 110, 253));
        btn_generate.set_label_color(Color::White);

        generator.fixed(&gen_v1_label, 90);
        generator.fixed(&gen_v1_from, 90);
        generator.fixed(&gen_v1_to, 90);
        generator.fixed(&gen_v1_step, 90);
        generator.fixed(&gen_sep, 12);
        generator.fixed(&gen_v2_label, 90);
        generator.fixed(&gen_v2_from, 90);
        generator.fixed(&gen_v2_to, 90);
        generator.fixed(&gen_v2_step, 90);
        generator.fixed(&btn_generate, 150);
        generator.end();

        let mut body = Flex::default().row();
        body.set_pad(14);

        let mut input_area = MultilineInput::default();
        input_area.set_trigger(CallbackTrigger::Changed);

        let mut result_table = Table::default();
        result_table.set_rows(0);
        result_table.set_row_header(true);
        result_table.set_row_header_width(64);
        result_table.set_cols(TABLE_HEADERS.len() as i32);
        result_table.set_col_header(true);
        result_table.set_col_width_all(118);
        result_table.set_col_resize(true);
        result_table.set_row_resize(false);
        result_table.end();

        let mut right = Flex::default().column();
        right.set_pad(10);

        let mut saved_title = Frame::default()
            .with_size(0, 28)
            .with_label("Сохраненные таблицы");
        saved_title.set_label_font(Font::HelveticaBold);
        saved_title.set_align(Align::Left | Align::Inside);

        let mut saved_browser = HoldBrowser::default();
        saved_browser.set_text_size(13);

        let mut saved_actions = Flex::default().with_size(0, 38).row();
        saved_actions.set_pad(10);
        let mut btn_load = Button::default().with_label("Загрузить");
        let mut btn_delete = Button::default().with_label("Удалить");
        btn_delete.set_color(Color::from_rgb(220, 53, 69));
        btn_delete.set_label_color(Color::White);
        saved_actions.fixed(&btn_load, 120);
        saved_actions.fixed(&btn_delete, 120);
        saved_actions.end();

        right.end();

        body.fixed(&input_area, 260);
        body.fixed(&right, 240);
        body.end();

        root.end();
        group.end();

        let model = Rc::new(RefCell::new(TableDisplayModel::default()));

        result_table.draw_cell({
            let model = model.clone();
            move |_, ctx, row, col, x, y, w, h| match ctx {
                TableContext::StartPage => draw::set_font(Font::Helvetica, 13),
                TableContext::ColHeader => {
                    let highlighted = model.borrow().selected_col == Some(col as usize);
                    draw_header(TABLE_HEADERS[col as usize], x, y, w, h, highlighted);
                }
                TableContext::RowHeader => {
                    let guard = model.borrow();
                    if let Some(row_data) = guard.rows.get(row as usize) {
                        let highlighted = guard.selected_row == Some(row as usize);
                        draw_header(&row_data.line_no.to_string(), x, y, w, h, highlighted);
                    }
                }
                TableContext::Cell => {
                    let guard = model.borrow();
                    if let Some(row_data) = guard.rows.get(row as usize) {
                        let selected =
                            guard.selected_row == Some(row as usize) || guard.selected_col == Some(col as usize);
                        let (bg, fg, text) = if let Some(error) = &row_data.error {
                            let bg = if selected {
                                Color::from_rgb(248, 215, 218)
                            } else {
                                Color::from_rgb(255, 245, 245)
                            };
                            let text = if col == 0 {
                                format!("Ошибка: {error}")
                            } else {
                                String::new()
                            };
                            (bg, Color::from_rgb(132, 32, 41), text)
                        } else {
                            let bg = if selected {
                                Color::from_rgb(230, 244, 255)
                            } else {
                                Color::White
                            };
                            let text = row_data.values.get(col as usize).cloned().unwrap_or_default();
                            (bg, Color::Black, text)
                        };
                        draw_table_cell(&text, x, y, w, h, bg, fg);
                    }
                }
                _ => {}
            }
        });

        result_table.set_callback({
            let model = model.clone();
            let mut table = result_table.clone();
            move |t| {
                let row = t.callback_row();
                let col = t.callback_col();
                let context = t.callback_context();
                let mut model = model.borrow_mut();
                match context {
                    TableContext::RowHeader if row >= 0 => {
                        let row = row as usize;
                        model.selected_row = if model.selected_row == Some(row) {
                            None
                        } else {
                            Some(row)
                        };
                        model.selected_col = None;
                    }
                    TableContext::ColHeader if col >= 0 => {
                        let col = col as usize;
                        model.selected_col = if model.selected_col == Some(col) {
                            None
                        } else {
                            Some(col)
                        };
                        model.selected_row = None;
                    }
                    TableContext::Cell if row >= 0 => {
                        let row = row as usize;
                        model.selected_row = if model.selected_row == Some(row) {
                            None
                        } else {
                            Some(row)
                        };
                        model.selected_col = None;
                    }
                    _ => {}
                }
                table.redraw();
            }
        });

        choice_mode.set_callback({
            let s = sender.clone();
            let input_area = input_area.clone();
            move |c| {
                s.send(Message::BatchDataChanged {
                    mode: choice_index_to_mode(c.value()),
                    content: input_area.value(),
                });
            }
        });

        choice_precision.set_callback({
            let s = sender.clone();
            move |c| {
                s.send(Message::SetTablePrecision(c.value().max(0) as usize));
            }
        });

        input_area.set_callback({
            let s = sender.clone();
            let input_area = input_area.clone();
            let choice_mode = choice_mode.clone();
            move |_| {
                s.send(Message::BatchDataChanged {
                    mode: choice_index_to_mode(choice_mode.value()),
                    content: input_area.value(),
                });
            }
        });

        btn_open.set_callback({
            let s = sender.clone();
            move |_| s.send(Message::LoadBatchFile)
        });

        btn_export.set_callback({
            let s = sender.clone();
            move |_| s.send(Message::ExportBatchData)
        });

        btn_clear.set_callback({
            let model = model.clone();
            let mut table = result_table.clone();
            move |_| {
                let mut model = model.borrow_mut();
                model.selected_row = None;
                model.selected_col = None;
                table.redraw();
            }
        });

        btn_save.set_callback({
            let s = sender.clone();
            let save_name = save_name.clone();
            let input_area = input_area.clone();
            let choice_mode = choice_mode.clone();
            move |_| {
                s.send(Message::SaveBatchTable {
                    name: save_name.value(),
                    mode: choice_index_to_mode(choice_mode.value()),
                    content: input_area.value(),
                });
            }
        });

        btn_generate.set_callback({
            let s = sender.clone();
            let gen_v1_from = gen_v1_from.clone();
            let gen_v1_to = gen_v1_to.clone();
            let gen_v1_step = gen_v1_step.clone();
            let gen_v2_from = gen_v2_from.clone();
            let gen_v2_to = gen_v2_to.clone();
            let gen_v2_step = gen_v2_step.clone();
            move |_| {
                s.send(Message::GenerateBatchData {
                    v1_from: gen_v1_from.value(),
                    v1_to: gen_v1_to.value(),
                    v1_step: gen_v1_step.value(),
                    v2_from: gen_v2_from.value(),
                    v2_to: gen_v2_to.value(),
                    v2_step: gen_v2_step.value(),
                });
            }
        });

        btn_load.set_callback({
            let s = sender.clone();
            let browser = saved_browser.clone();
            move |_| {
                let index = browser.value();
                if index > 0 {
                    s.send(Message::LoadSavedTable((index - 1) as usize));
                }
            }
        });

        btn_delete.set_callback({
            let s = sender.clone();
            let browser = saved_browser.clone();
            move |_| {
                let index = browser.value();
                if index > 0 {
                    s.send(Message::DeleteSavedTable((index - 1) as usize));
                }
            }
        });

        saved_browser.set_callback({
            let s = sender.clone();
            move |browser| {
                let index = browser.value();
                if app::event_clicks() && index > 0 {
                    s.send(Message::LoadSavedTable((index - 1) as usize));
                }
            }
        });

        Self {
            group,
            choice_mode,
            choice_precision,
            input_area,
            save_name,
            saved_browser,
            gen_v1_from,
            gen_v1_to,
            gen_v1_step,
            gen_v2_from,
            gen_v2_to,
            gen_v2_step,
            gen_v1_label,
            gen_v2_label,
            result_table,
            model,
        }
    }

    /// Устанавливает режим расчета и обновляет подписи генератора.
    pub fn set_mode(&mut self, mode: InputMode) {
        self.choice_mode.set_value(mode_to_choice_index(mode));
        let (label_a, label_b) = mode_labels(mode);
        self.gen_v1_label.set_label(label_a);
        self.gen_v2_label.set_label(label_b);
    }

    /// Устанавливает точность вывода (0..=10 знаков после запятой).
    pub fn set_precision(&mut self, precision: usize) {
        self.choice_precision.set_value(precision.min(10) as i32);
    }

    /// Устанавливает текст табличного ввода.
    pub fn set_input(&mut self, text: &str) {
        self.input_area.set_value(text);
    }

    /// Устанавливает имя для сохранения таблицы.
    pub fn set_save_name(&mut self, value: &str) {
        self.save_name.set_value(value);
    }

    /// Синхронизирует список сохраненных таблиц.
    pub fn sync_saved_tables(&mut self, labels: &[String]) {
        self.saved_browser.clear();
        for label in labels {
            self.saved_browser.add(label);
        }
    }

    /// Обновляет модель отображения и перерисовывает таблицу результата.
    pub fn update_table(&mut self, rows: &[BatchRow], precision: usize) {
        let display_rows = rows
            .iter()
            .map(|row| {
                let values = row
                    .state
                    .as_ref()
                    .map(|state| {
                        vec![
                            format_value(state.p.inner(), precision),
                            format_value(state.t.inner(), precision),
                            format_value(state.v.inner(), precision),
                            format_value(state.rho.inner(), precision),
                            format_value(state.h.inner(), precision),
                            format_value(state.s.inner(), precision),
                            format_value(state.u.inner(), precision),
                            format_value(state.cp.inner(), precision),
                            format_value(state.w.inner(), precision),
                            format!("{:?}", state.region),
                        ]
                    })
                    .unwrap_or_default();

                DisplayRow {
                    line_no: row.line_no,
                    values,
                    error: row.error.clone(),
                }
            })
            .collect::<Vec<_>>();

        let mut model = self.model.borrow_mut();
        model.rows = display_rows;
        model.selected_row = None;
        model.selected_col = None;
        drop(model);

        self.result_table.set_rows(rows.len() as i32);
        self.result_table.redraw();
    }
}
