use if97_app_api::LogEntryDto;
use once_cell::sync::Lazy;
use std::collections::VecDeque;
use std::sync::Mutex;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

const MAX_LOG_ENTRIES: usize = 2000;

pub static LOG_BUFFER: Lazy<Mutex<VecDeque<LogEntryDto>>> =
    Lazy::new(|| Mutex::new(VecDeque::with_capacity(MAX_LOG_ENTRIES)));

struct WasmLayer;

impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for WasmLayer {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let metadata = event.metadata();
        let target = metadata.target();

        if target.starts_with("yew")
            || target.starts_with("gloo")
            || target.starts_with("wasm_bindgen")
        {
            return;
        }

        if *metadata.level() == tracing::Level::TRACE || *metadata.level() == tracing::Level::DEBUG
        {
            return;
        }

        let mut visitor = StringVisitor {
            message: String::new(),
        };
        event.record(&mut visitor);

        if visitor.message.trim().is_empty() {
            return;
        }

        if let Ok(mut buffer) = LOG_BUFFER.lock() {
            if buffer.len() == MAX_LOG_ENTRIES {
                buffer.pop_front();
            }
            buffer.push_back(LogEntryDto {
                source: "frontend".to_string(),
                level: metadata.level().to_string().to_lowercase(),
                message: format!("[frontend][{}] {}", metadata.level(), visitor.message),
            });
        }
    }
}

struct StringVisitor {
    message: String,
}

impl tracing::field::Visit for StringVisitor {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        }
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" && self.message.is_empty() {
            self.message = format!("{value:?}");
        }
    }
}

pub fn snapshot_logs() -> Vec<LogEntryDto> {
    LOG_BUFFER
        .lock()
        .map(|buffer| buffer.iter().cloned().collect())
        .unwrap_or_default()
}

pub fn clear_logs() {
    if let Ok(mut buffer) = LOG_BUFFER.lock() {
        buffer.clear();
    }
}

pub fn init_logger() {
    let subscriber = Registry::default().with(WasmLayer);
    let _ = tracing::subscriber::set_global_default(subscriber);
}
