use crate::config::AppConfig;
use crate::pmp_api::PmpClient;
use tokio::sync::Mutex;

/// 全局应用状态
pub struct AppState {
    pub config: Mutex<Option<AppConfig>>,
    pub client: Mutex<Option<PmpClient>>,
    pub config_error: Mutex<Option<String>>,
}
