use std::{path::PathBuf, sync::Arc};

use chrono::Utc;
use clap::Parser;
use tokio::fs;
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;

use crate::{
    config::AppConfig,
    errors::{Error, Result},
    gate::daemon::AppDaemon,
    state::AppState,
};

mod config;
mod errors;
mod state;
mod gate {
    pub mod daemon;
    pub mod limiter;
    pub mod relay;
}

#[derive(Debug, Parser)]
struct Args {
    #[arg(long)]
    config_file: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let logger_worker = init_logger().await?;

    let args = Args::parse();

    tracing::info!("App start!");
    tracing::info!("Reading config file...");

    let config_file = match args.config_file {
        Some(f) => f,
        None => PathBuf::from("craft-gate/config.toml"),
    };
    let cfg = get_cfg(config_file).await?;
    let app_state = AppState {
        _logger_worker: logger_worker,
        config: Arc::new(cfg),
    };

    let daemon = AppDaemon::new(app_state);
    daemon.start().await?;

    tracing::info!("Loaded config file");

    tracing::info!("End of app");
    Ok(())
}

async fn get_cfg(config_file: PathBuf) -> Result<AppConfig> {
    let ret = match AppConfig::create_from_file(&config_file).await {
        Ok(ret) => ret,
        Err(e) => {
            tracing::warn!("Can not read config file, use default data: {e}");
            if fs::try_exists(&config_file).await.map_err(Error::IO)? {
                let t = Utc::now().timestamp();
                fs::rename(&config_file, config_file.with_extension(format!("bak{t}")))
                    .await
                    .map_err(Error::IO)?;
            } else {
                AppConfig::default().write_to_file(&config_file).await?;
            }

            AppConfig::default()
        }
    };

    Ok(ret)
}

async fn init_logger() -> Result<WorkerGuard> {
    let log_dir = PathBuf::from("craft-gate/logs");
    fs::create_dir_all(&log_dir).await.map_err(Error::IO)?;
    let file_appender = tracing_appender::rolling::daily(&log_dir, "");
    let (non_block, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_writer(std::io::stdout)
        .with_writer(non_block)
        .with_ansi(false)
        .init();

    Ok(guard)
}
