use std::fs;
use tauri::{AppHandle, Manager};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tracing_appender::non_blocking::WorkerGuard;

pub fn init(handle: &AppHandle) -> Result<WorkerGuard, Box<dyn std::error::Error>> {
    let log_dir = handle.path().app_log_dir().map_err(|_| "Could not find log directory")?;
    if !log_dir.exists() {
        fs::create_dir_all(&log_dir)?;
    }
    let file_appender = tracing_appender::rolling::daily(&log_dir, "app.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::Layer::default().with_writer(std::io::stdout)) // Dev console
        .with(fmt::Layer::default().with_writer(non_blocking))   // File
        .try_init()?;

    tracing::info!("Logging initialized in: {:?}", log_dir);
    Ok(guard)
}