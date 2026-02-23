use std::fs;
use tauri::{AppHandle, Manager};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tracing_appender::non_blocking::WorkerGuard;

pub fn init(handle: &AppHandle) -> Result<WorkerGuard, Box<dyn std::error::Error>> {
    let log_dir = handle.path().app_log_dir()?;
    if !log_dir.exists() {
        fs::create_dir_all(&log_dir)?;
    }
    let file_appender = tracing_appender::rolling::daily(&log_dir, "app.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        // Log level configuration (default is INFO)
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        // Console output
        .with(fmt::Layer::default().with_writer(std::io::stdout))
        // File output
        .with(fmt::Layer::default().with_writer(non_blocking))
        .try_init()?;

    tracing::info!("Logging initialized. Logs are saved in: {:?}", log_dir);
    Ok(guard)
}