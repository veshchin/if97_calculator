// File: src/ui/batch.rs

use fltk::{prelude::*, group::*, button::*, input::*, menu::*, frame::*, browser::HoldBrowser, enums::*};
use fltk::app::Sender;
use crate::state::Message;
use tracing::{info, debug};

pub struct BatchTab {
    pub group: Group,
    pub choice_mode: Choice,
    pub input_area: MultilineInput,
    pub output_table: HoldBrowser,
}

impl BatchTab {
    pub fn new(sender: Sender<Message>) -> Self {
        // 1. Создаем основную группу вкладки
        let mut group = Group::new(10, 35, 1030, 655, " Табличный расчет ");

        // 2. Инструкция
        let mut instruction = Frame::new(20, 45, 1010, 40, "Инструкция: Вставьте данные в левое поле или загрузите файл. Разделители: пробел, табуляция, запятая. Дробная часть — ТОЧКА.");
        instruction.set_align(Align::Left | Align::Inside);
        instruction.set_label_color(Color::Dark3);
        instruction.set_label_font(Font::HelveticaItalic);

        // 3. Панель управления (кнопки и выбор режима)
        let mut btn_load = Button::new(20, 85, 120, 30, "Загрузить файл");

        let mut btn_save = Button::new(150, 85, 150, 30, "В графики");
        btn_save.set_color(Color::from_rgb(34, 139, 34));
        btn_save.set_label_color(Color::White);

        // Кнопка экспорта (теперь ВНУТРИ группы)
        let mut btn_export = Button::new(310, 85, 140, 30, "Экспорт в файл");
        btn_export.set_color(Color::from_rgb(70, 130, 180)); // Стальной синий
        btn_export.set_label_color(Color::White);

        let mut choice_mode = Choice::new(520, 85, 200, 30, "Вход:");
        choice_mode.add_choice("p, T|rho, T|p, h|p, s|p, x");
        choice_mode.set_value(0);

        // 4. Основная область данных (Flex для гибкой разметки)
        let mut flex = Flex::new(20, 125, 1000, 520, "").row();
        flex.set_pad(20);

        let mut input_area = MultilineInput::default().with_label("Ввод данных (2 колонки):");
        input_area.set_align(Align::TopLeft);

        let mut output_table = HoldBrowser::default().with_label("Результат:");
        output_table.set_align(Align::TopLeft);
        output_table.set_text_size(13);
        // Колонки: P, T, Reg, x, v, rho, h, s, u, cp, cv, w
        output_table.set_column_widths(&[75, 65, 50, 60, 85, 75, 80, 70, 80, 70, 60, 70, 0]); //        output_table.set_column_char('\t');

        flex.fixed(&input_area, 180); // Левая панель ввода фиксированной ширины
        flex.end();

        // Завершаем создание группы
        group.end();

        // --- Коллбеки (Логика) ---

        btn_load.set_callback({
            let s = sender.clone();
            move |_| s.send(Message::LoadBatchFile)
        });

        btn_save.set_callback({
            let s = sender.clone();
            move |_| s.send(Message::SaveBatchTable)
        });

        btn_export.set_callback({
            let s = sender.clone();
            move |_| s.send(Message::ExportBatchData)
        });

        input_area.set_callback({
            let s = sender.clone();
            let cm = choice_mode.clone();
            move |i| {
                debug!("Пакетные данные изменены, отправка на расчет. Режим: {}", cm.value());
                s.send(Message::BatchDataChanged { mode: cm.value(), content: i.value() })
            }
        });
        input_area.set_trigger(CallbackTrigger::Changed);

        choice_mode.set_callback({
            let mut ia = input_area.clone();
            move |c| {
                info!("Смена режима пакетного расчета на {}", c.value());
                ia.do_callback();
            }
        });

        Self { group, choice_mode, input_area, output_table }
    }

    pub fn set_input(&mut self, text: &str) {
        self.input_area.set_value(text);
        self.input_area.do_callback(); // Принудительно триггерим пересчет
    }
}