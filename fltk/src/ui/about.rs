// File: src/ui/about.rs

use crate::state::Message;
use fltk::app::Sender;
use fltk::{button::*, enums::*, frame::*, group::*, prelude::*};
use tracing::{error, info};

pub struct AboutTab {
    pub group: Group,
}

impl AboutTab {
    pub fn new(sender: Sender<Message>) -> Self {
        // Создаем группу для вкладки "О программе" [cite: 706, 707]
        let group = Group::new(10, 35, 1030, 655, " О программе ");

        let mut pack = Pack::new(40, 60, 950, 600, "");
        pack.set_spacing(20);

        // Заголовок программы
        let mut title = Frame::default()
            .with_size(0, 40)
            .with_label("IAPWS-IF97 Calculator Pro");
        title.set_label_font(Font::HelveticaBold);
        title.set_label_size(24);

        // Информация об авторе и ядре
        let mut about_text = Frame::default().with_size(0, 100).with_label(
            "Профессиональный инструмент для теплотехнических расчетов.\n\
             Ядро реализовано на Rust с использованием строгой типизации размерностей.\n\
             Автор: [Ваш Никнейм / Имя]",
        );
        about_text.set_align(Align::Left | Align::Inside);

        // Кнопки ссылок
        let mut btn_flex = Flex::default().with_size(0, 40).row();
        btn_flex.set_pad(10);

        let mut btn_github = Button::default().with_label("@ GitHub Project");
        btn_github.set_callback(|_| {
            if let Err(e) = webbrowser::open("https://github.com/veshchin/if97_calculator") {
                error!("Не удалось открыть ссылку GitHub: {}", e);
            }
        });

        let mut btn_donate = Button::default().with_label("☕ Поддержать проект");
        btn_donate.set_color(Color::from_rgb(255, 165, 0));
        btn_donate.set_label_color(Color::White);
        btn_donate.set_callback(|_| {
            if let Err(e) = webbrowser::open("https://donationalerts.com/r/your_page") {
                error!("Не удалось открыть ссылку доната: {}", e);
            }
        });

        btn_flex.fixed(&btn_github, 180);
        btn_flex.fixed(&btn_donate, 200);
        btn_flex.end();

        // Системный раздел
        Frame::default().with_size(0, 20); // Отступ
        let mut sys_label = Frame::default()
            .with_size(0, 30)
            .with_label("Сервис и диагностика:");
        sys_label.set_align(Align::Left | Align::Inside);
        sys_label.set_label_font(Font::HelveticaItalic);

        let mut btn_save_logs = Button::default()
            .with_size(0, 40)
            .with_label("Сохранить логи работы (.log)");
        btn_save_logs.set_color(Color::from_rgb(100, 100, 100));
        btn_save_logs.set_label_color(Color::White);

        // Отправка сообщения в главный цикл для сохранения файла [cite: 593, 594]
        btn_save_logs.set_callback({
            let s = sender.clone();
            move |_| {
                info!("Запрос на экспорт логов из вкладки About");
                s.send(Message::SaveLogFile);
            }
        });

        pack.end();
        group.end();

        Self { group }
    }
}
