use std::{path::PathBuf, sync::Arc};

use chrono::Utc;
use clap::Parser;
use tokio::fs;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::writer::MakeWriterExt;

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

    #[arg(long, default_value_t = false)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let logger_worker = init_logger(if args.debug {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    })
    .await?;

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

async fn init_logger(logger_level: tracing::Level) -> Result<Vec<WorkerGuard>> {
    // file
    let log_dir = PathBuf::from("craft-gate/logs");
    fs::create_dir_all(&log_dir).await.map_err(Error::IO)?;
    let file_appender = tracing_appender::rolling::daily(&log_dir, "");
    let (non_block, guard) = tracing_appender::non_blocking(file_appender);

    // std
    let (non_block_std, std_guard) = tracing_appender::non_blocking(std::io::stdout());

    let writer = non_block_std.and(non_block);
    tracing_subscriber::fmt()
        .with_max_level(logger_level)
        .with_writer(writer)
        .with_ansi(false)
        .init();

    Ok(vec![guard, std_guard])
}
