use std::path::Path;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use tokio::fs;
use url::Url;

use crate::errors::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub listen: Url,
    pub ip_limit: usize,
    pub services: AppConfigService,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            listen: Url::parse("tcp://0.0.0.0:25565").unwrap(),
            ip_limit: 64,
            services: AppConfigService::default(),
        }
    }
}

impl AppConfig {
    pub async fn create_from_file<P: AsRef<Path>>(file_path: P) -> Result<Self> {
        let content = fs::read_to_string(file_path).await.map_err(Error::IO)?;

        let config: AppConfig = toml::from_str(&content).map_err(|e| {
            Error::Serialization(anyhow!(format!(
                "Failed to parse config file as TOML: {0}",
                e
            )))
        })?;

        Ok(config)
    }

    pub async fn write_to_file<P: AsRef<Path>>(self, file_path: P) -> Result<()> {
        let content = toml::to_string(&self)
            .map_err(|e| anyhow!(format!("Failed to deserialize config file: {e}")))?;
        fs::write(file_path, content.as_bytes())
            .await
            .map_err(Error::IO)?;

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfigService {
    pub minecraft: AppConfigServiceMinecraft,
    pub http: AppConfigServiceHttp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfigServiceMinecraft {
    pub dest: Url,
}

impl Default for AppConfigServiceMinecraft {
    fn default() -> Self {
        AppConfigServiceMinecraft {
            dest: Url::parse("tcp://127.0.0.1:11451").unwrap(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfigServiceHttp {
    pub dest: Url,
    pub mode: AppConfigServiceHttpMode,
}

impl Default for AppConfigServiceHttp {
    fn default() -> Self {
        Self {
            dest: Url::parse("tcp://127.0.0.1:8080").unwrap(),
            mode: AppConfigServiceHttpMode::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum AppConfigServiceHttpMode {
    #[default]
    Proxy,
    Redirect,
}
