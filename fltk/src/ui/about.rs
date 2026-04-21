//! UI-вкладка "О программе" и экспорт логов.

use crate::state::Message;
use fltk::app::Sender;
use fltk::{button::*, enums::*, frame::*, group::*, prelude::*};
use tracing::{error, info};

#[allow(dead_code)]
/// Вкладка UI с информацией о приложении и системными действиями.
pub struct AboutTab {
    /// Корневой контейнер вкладки.
    pub group: Group,
}

impl AboutTab {
    /// Создает вкладку и настраивает callback'и для отправки [`Message`] в обработчик.
    pub fn new(sender: Sender<Message>) -> Self {
        let group = Group::new(10, 35, 1030, 655, " О программе ");

        let mut pack = Pack::new(40, 60, 950, 600, "");
        pack.set_spacing(20);

        let mut title = Frame::default().with_size(0, 40).with_label("IF97 Calculator");
        title.set_label_font(Font::HelveticaBold);
        title.set_label_size(24);

        let mut about_text = Frame::default().with_size(0, 100).with_label(
            "Версия 1.1.0\n\
             Кроссплатформенный калькулятор свойств воды и пара по IAPWS-IF97.\n\
             Ядро, desktop UI, Tauri UI и service-обертка используют общую вычислительную модель.",
        );
        about_text.set_align(Align::Left | Align::Inside);

        let mut btn_flex = Flex::default().with_size(0, 40).row();
        btn_flex.set_pad(10);

        let mut btn_github = Button::default().with_label("Открыть репозиторий");
        btn_github.set_callback(|_| {
            if let Err(e) = webbrowser::open("https://github.com/veshchin/if97_calculator") {
                error!("Не удалось открыть ссылку GitHub: {}", e);
            }
        });

        btn_flex.fixed(&btn_github, 180);
        btn_flex.end();

        Frame::default().with_size(0, 20);
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
