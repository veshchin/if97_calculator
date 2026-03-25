use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_subscriber() {
    // Настраиваем логирование с дефолтным уровнем INFO
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "if97_core=info,tonic=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}