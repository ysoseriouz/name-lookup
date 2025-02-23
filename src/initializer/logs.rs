use std::{
    fs,
    time::{Duration, SystemTime},
};

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
        .with(tracing_subscriber::fmt::layer().with_level(true).compact())
        .init();

    tokio::spawn(rotate_logs());
}

pub async fn rotate_logs() {
    if let Err(err) = archive_logs("logs", 7) {
        tracing::error!("Rotate log failed: {:?}", err);
    }
}

fn archive_logs(log_dir: &str, days: u64) -> std::io::Result<()> {
    let now = SystemTime::now();
    let cutoff_time = now - Duration::from_secs(days * 86400);

    for entry in fs::read_dir(log_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        if let Ok(metadata) = fs::metadata(&path) {
            if let Ok(modified_time) = metadata.modified() {
                if modified_time < cutoff_time {
                    tracing::info!("Archive old log file: {:?}", path);
                    // TODO: replace this
                    fs::remove_file(path)?;
                }
            }
        }
    }

    Ok(())
}
