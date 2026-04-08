use tracing_subscriber::EnvFilter;

/// Инициализирует `tracing` subscriber для сервиса.
///
/// Поддерживает настройку через `RUST_LOG`. Если переменная не задана, используется
/// консервативный default-фильтр.
pub fn init() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,if97_calculator_service=info"));

    tracing_subscriber::fmt().with_env_filter(filter).init();
}
