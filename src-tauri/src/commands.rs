use crate::pmp_api;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{Manager, State};

/// 配置摘要（前端展示用）
#[derive(Serialize, Clone)]
pub struct ConfigResponse {
    pub username: String,
    pub password: String,
    pub default_hours: f64,
    pub save_or_submit: String,
    pub config_path: String,
    pub today: String,
    pub config_error: Option<String>,
    pub logged_in: bool,
    pub base_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub default_task_desc: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub ok: bool,
    pub user_id: Option<String>,
    pub expires_at: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct TasksResponse {
    pub list: Vec<pmp_api::TaskItem>,
}

#[derive(Serialize)]
pub struct TimesheetsResponse {
    pub list: Vec<pmp_api::TimesheetRecord>,
}

#[derive(Serialize)]
pub struct SubmitResponse {
    pub ok: bool,
    pub result: Option<Value>,
    pub error: Option<String>,
}

#[derive(Deserialize)]
pub struct SubmitRequest {
    pub day: String,
    pub task_ids: Vec<String>,
    pub task_desc: String,
    pub hours: f64,
    pub save_or_submit: Option<String>,
}

/// 登录请求（来自登录页面）
#[derive(Deserialize)]
pub struct LoginWithConfigRequest {
    pub base_url: Option<String>,
    pub username: String,
    pub password: String,
    pub client_id: String,
    pub client_secret: String,
    pub default_hours: Option<f64>,
    pub save_or_submit: Option<String>,
    pub default_task_desc: Option<String>,
}

/// 切换 DevTools（仅 debug 模式可用）
#[tauri::command]
#[cfg(debug_assertions)]
pub fn toggle_devtools(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_devtools_open() {
            window.close_devtools();
        } else {
            window.open_devtools();
        }
    }
}

/// 切换 DevTools（release 模式空实现）
#[tauri::command]
#[cfg(not(debug_assertions))]
pub fn toggle_devtools(_app: tauri::AppHandle) {}

/// 最小化窗口
#[tauri::command]
pub fn minimize_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.minimize();
    }
}

/// 关闭窗口
#[tauri::command]
pub fn close_window(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.close();
    }
}

/// 获取配置摘要
#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<ConfigResponse, String> {
    let today = pmp_api::today_str();
    let config = state.config.lock().await.clone();
    let config_error = state.config_error.lock().await.clone();

    if let Some(ref cfg) = config {
        let logged_in = {
            let client = state.client.lock().await;
            client.as_ref().map(|c| c.is_logged_in()).unwrap_or(false)
        };

        Ok(ConfigResponse {
            username: cfg.username.clone(),
            password: cfg.password.clone(),
            default_hours: cfg.default_hours,
            save_or_submit: cfg.save_or_submit.clone(),
            config_path: cfg.config_path.clone(),
            today,
            config_error,
            logged_in,
            base_url: cfg.base_url.clone(),
            client_id: cfg.client_id.clone(),
            client_secret: cfg.client_secret.clone(),
            default_task_desc: cfg.default_task_desc.clone(),
        })
    } else {
        Err(config_error.unwrap_or_else(|| "配置未加载".to_string()))
    }
}

/// 登录（使用已有配置）
#[tauri::command]
pub async fn login(state: State<'_, AppState>) -> Result<LoginResponse, String> {
    let mut client = state.client.lock().await;
    match client.as_mut() {
        Some(c) => match c.login().await {
            Ok(info) => Ok(LoginResponse {
                ok: true,
                user_id: info.user_id,
                expires_at: info.expires_at,
                error: None,
            }),
            Err(e) => Ok(LoginResponse {
                ok: false,
                user_id: None,
                expires_at: None,
                error: Some(e),
            }),
        },
        None => Err("PMP 客户端未初始化，请先登录".to_string()),
    }
}

/// 使用新凭据登录并保存配置
#[tauri::command]
pub async fn login_with_config(
    state: State<'_, AppState>,
    req: LoginWithConfigRequest,
) -> Result<LoginResponse, String> {
    use crate::config::AppConfig;

    let base_url = req
        .base_url
        .clone()
        .unwrap_or_else(|| "http://localhost:8080".to_string())
        .trim_end_matches('/')
        .to_string();

    let config = AppConfig {
        base_url,
        username: req.username.clone(),
        password: req.password.clone(),
        client_id: req.client_id.clone(),
        client_secret: req.client_secret.clone(),
        grant_type: "password".to_string(),
        scope: "server".to_string(),
        default_hours: req.default_hours.unwrap_or(7.5),
        save_or_submit: req.save_or_submit.clone().unwrap_or_else(|| "2".to_string()),
        default_task_desc: req.default_task_desc.clone().unwrap_or_default(),
        config_path: String::new(),
    };

    // 创建新的 PMP client 并登录（登录成功后再持久化，避免错误凭据被写入配置）
    let mut client = pmp_api::PmpClient::new(config.clone());
    match client.login().await {
        Ok(info) => {
            let _ = crate::config::save_config_to_file(&config);
            // 更新全局状态
            let mut state_client = state.client.lock().await;
            *state_client = Some(client);
            drop(state_client);

            // 更新 config
            let mut config_with_path = config;
            config_with_path.config_path =
                crate::config::default_config_path().to_string_lossy().to_string();
            *state.config.lock().await = Some(config_with_path);
            *state.config_error.lock().await = None;

            Ok(LoginResponse {
                ok: true,
                user_id: info.user_id,
                expires_at: info.expires_at,
                error: None,
            })
        }
        Err(e) => Ok(LoginResponse {
            ok: false,
            user_id: None,
            expires_at: None,
            error: Some(e),
        }),
    }
}

/// 强制重新登录
#[tauri::command]
pub async fn relogin(state: State<'_, AppState>) -> Result<LoginResponse, String> {
    login(state).await
}

/// 查询任务列表
#[tauri::command]
pub async fn list_tasks(
    state: State<'_, AppState>,
    page_no: Option<u32>,
    page_size: Option<u32>,
) -> Result<TasksResponse, String> {
    let mut client = state.client.lock().await;
    match client.as_mut() {
        Some(c) => {
            let list = c
                .list_doing_tasks(page_no.unwrap_or(1), page_size.unwrap_or(50))
                .await?;
            Ok(TasksResponse { list })
        }
        None => Err("PMP 客户端未初始化".to_string()),
    }
}

/// 查询工时记录
#[tauri::command]
pub async fn list_timesheets(
    state: State<'_, AppState>,
    page_no: Option<u32>,
    page_size: Option<u32>,
) -> Result<TimesheetsResponse, String> {
    let mut client = state.client.lock().await;
    match client.as_mut() {
        Some(c) => {
            let list = c
                .list_timesheets(page_no.unwrap_or(1), page_size.unwrap_or(31))
                .await?;
            Ok(TimesheetsResponse { list })
        }
        None => Err("PMP 客户端未初始化".to_string()),
    }
}

/// 提交工时
#[tauri::command]
pub async fn submit_timesheet(
    state: State<'_, AppState>,
    req: SubmitRequest,
) -> Result<SubmitResponse, String> {
    let mut client = state.client.lock().await;
    match client.as_mut() {
        Some(c) => match c
            .submit_timesheet(
                &req.day,
                &req.task_ids,
                &req.task_desc,
                req.hours,
                req.save_or_submit,
            )
            .await
        {
            Ok(result) => Ok(SubmitResponse {
                ok: true,
                result: Some(result),
                error: None,
            }),
            Err(e) => Ok(SubmitResponse {
                ok: false,
                result: None,
                error: Some(e),
            }),
        },
        None => Err("PMP 客户端未初始化".to_string()),
    }
}
