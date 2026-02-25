use std::sync::Arc;

use tracing_appender::non_blocking::WorkerGuard;

use crate::config::AppConfig;

pub struct AppState {
    pub _logger_worker: Vec<WorkerGuard>,
    pub config: Arc<AppConfig>,
}
