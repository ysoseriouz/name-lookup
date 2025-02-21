use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn setup_logs() {
    let log_file = tracing_appender::rolling::daily("logs", "app.log");
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or("DEBUG".into()))
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(log_file)
                .with_target(false)
                .with_level(true)
                .json()
                .compact(),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_level(true)
                .json()
                .compact(),
        )
        .init();
}
