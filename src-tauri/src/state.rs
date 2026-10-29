use crate::config::{AppConfig, ConfigIssue};
use crate::pmp_api::PmpClient;
use tokio::sync::Mutex;

/// 全局应用状态
pub struct AppState {
    pub config: Mutex<Option<AppConfig>>,
    pub client: Mutex<Option<PmpClient>>,
    /// 配置缺失/不完整/损坏的原因；None 表示配置可用
    pub config_issue: Mutex<Option<ConfigIssue>>,
}
