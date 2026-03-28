/* File: src/logger.rs */
use std::sync::Mutex;
use once_cell::sync::Lazy;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

pub static LOG_BUFFER: Lazy<Mutex<Vec<(String, String)>>> = Lazy::new(|| Mutex::new(Vec::new()));

struct WasmLayer;

impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for WasmLayer {
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let metadata = event.metadata();
        let target = metadata.target();

        // 1. Отсекаем мусорные логи от Yew, gloo и системных WASM-библиотек
        if target.starts_with("yew") || target.starts_with("gloo") || target.starts_with("wasm_bindgen") {
            return;
        }

        // 2. Оставляем только INFO, WARN и ERROR (отбрасываем TRACE и DEBUG)
        if *metadata.level() == tracing::Level::TRACE || *metadata.level() == tracing::Level::DEBUG {
            return;
        }

        let mut visitor = StringVisitor { message: String::new() };
        event.record(&mut visitor);

        // 3. Отбрасываем пустые сообщения
        if visitor.message.trim().is_empty() {
            return;
        }

        let level = metadata.level().to_string().to_lowercase();
        let msg = format!("[{}] {}", metadata.level(), visitor.message);

        if let Ok(mut buffer) = LOG_BUFFER.lock() {
            buffer.push((level, msg));
        }
    }
}

struct StringVisitor {
    message: String,
}

impl tracing::field::Visit for StringVisitor {
    // Перехватываем строки напрямую, чтобы они не оборачивались в кавычки
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        }
    }

    // Резервный вариант для остальных типов данных
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" && self.message.is_empty() {
            self.message = format!("{:?}", value);
        }
    }
}

pub fn init_logger() {
    let subscriber = Registry::default().with(WasmLayer);
    let _ = tracing::subscriber::set_global_default(subscriber);
}