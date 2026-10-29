use crate::config::AppConfig;
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
    /// 还没有可用配置（首次使用 / 配置不完整），前端应展示引导而不是报错
    pub needs_setup: bool,
    pub logged_in: bool,
    pub base_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub default_task_desc: String,
    /// 是否记住工作描述（提交后保留、下次自动填入）
    pub remember_task_desc: bool,
    /// 是否记住选中的任务（下次自动选中）
    pub remember_task_id: bool,
    /// 上次提交使用的工作描述
    pub last_task_desc: String,
    /// 上次提交选中的任务 ID
    pub last_task_id: String,
}

/// 偏好开关（配置管理页的「记住…」开关）
#[derive(Deserialize)]
pub struct PreferencesRequest {
    pub remember_task_desc: Option<bool>,
    pub remember_task_id: Option<bool>,
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
///
/// 配置缺失/不完整不再是错误：返回空配置 + needs_setup=true，
/// 让前端渲染「首次使用」引导，而不是把内部错误抛给用户。
#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<ConfigResponse, String> {
    let today = pmp_api::today_str();
    let config = state.config.lock().await.clone();
    let issue = state.config_issue.lock().await.clone();

    let logged_in = {
        let client = state.client.lock().await;
        client.as_ref().map(|c| c.is_logged_in()).unwrap_or(false)
    };

    let needs_setup = issue.as_ref().map(|i| i.needs_setup()).unwrap_or(false);
    // 缺配置时 message 是引导语（前端用蓝色提示展示），配置文件损坏时才是真正的错误
    let config_error = issue.as_ref().map(|i| i.message.clone());

    let cfg = config.unwrap_or_else(AppConfig::empty);

    Ok(ConfigResponse {
        username: cfg.username,
        password: cfg.password,
        default_hours: cfg.default_hours,
        save_or_submit: cfg.save_or_submit,
        config_path: cfg.config_path,
        today,
        config_error,
        needs_setup,
        logged_in,
        base_url: cfg.base_url,
        client_id: cfg.client_id,
        client_secret: cfg.client_secret,
        default_task_desc: cfg.default_task_desc,
        remember_task_desc: cfg.remember_task_desc,
        remember_task_id: cfg.remember_task_id,
        last_task_desc: cfg.last_task_desc,
        last_task_id: cfg.last_task_id,
    })
}

/// 保存「记住工作描述 / 记住选中任务」两个开关
///
/// 这两个是纯本地偏好，不涉及凭据，因此不重新登录、直接落盘。
/// 关闭时顺手清掉已记住的值，避免下次重新打开开关又冒出旧内容。
#[tauri::command]
pub async fn save_preferences(
    state: State<'_, AppState>,
    req: PreferencesRequest,
) -> Result<bool, String> {
    let snapshot = {
        let mut guard = state.config.lock().await;
        let Some(config) = guard.as_mut() else {
            // 还没有配置（首次使用）：没有可保存的地方，安静返回
            return Ok(false);
        };

        if !config.set_remember_flags(req.remember_task_desc, req.remember_task_id) {
            return Ok(true);
        }

        config.clone()
    };

    crate::config::save_config_to_file(&snapshot)?;
    Ok(true)
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
    // 服务器地址没有默认值，必须由用户填写
    let base_url = req
        .base_url
        .clone()
        .unwrap_or_default()
        .trim()
        .trim_end_matches('/')
        .to_string();

    if base_url.is_empty() {
        return Ok(LoginResponse {
            ok: false,
            user_id: None,
            expires_at: None,
            error: Some("请填写服务器地址".to_string()),
        });
    }

    if req.username.trim().is_empty() {
        return Ok(LoginResponse {
            ok: false,
            user_id: None,
            expires_at: None,
            error: Some("请填写用户名".to_string()),
        });
    }

    if req.password.is_empty() {
        return Ok(LoginResponse {
            ok: false,
            user_id: None,
            expires_at: None,
            error: Some("请填写密码".to_string()),
        });
    }

    if req.client_id.trim().is_empty() {
        return Ok(LoginResponse {
            ok: false,
            user_id: None,
            expires_at: None,
            error: Some("请填写 Client ID".to_string()),
        });
    }

    if req.client_secret.is_empty() {
        return Ok(LoginResponse {
            ok: false,
            user_id: None,
            expires_at: None,
            error: Some("请填写 Client Secret".to_string()),
        });
    }

    // 重新登录（登录页 / 配置管理）不涉及「记住…」这组偏好，
    // 因此沿用已有配置里的值，避免每次登录把它们重置成默认。
    let previous = state.config.lock().await.clone();
    let (remember_task_desc, remember_task_id, last_task_desc, last_task_id) = match previous {
        Some(c) => (
            c.remember_task_desc,
            c.remember_task_id,
            c.last_task_desc,
            c.last_task_id,
        ),
        None => (
            crate::config::default_true(),
            crate::config::default_true(),
            String::new(),
            String::new(),
        ),
    };

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
        remember_task_desc,
        remember_task_id,
        last_task_desc,
        last_task_id,
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
            *state.config_issue.lock().await = None;

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
    let outcome = {
        let mut client = state.client.lock().await;
        match client.as_mut() {
            Some(c) => {
                c.submit_timesheet(
                    &req.day,
                    &req.task_ids,
                    &req.task_desc,
                    req.hours,
                    req.save_or_submit.clone(),
                )
                .await
            }
            None => return Err("PMP 客户端未初始化".to_string()),
        }
    };

    match outcome {
        Ok(result) => {
            // 提交成功才更新「上次使用」：失败的内容不该被记住
            remember_last_used(&state, &req).await;
            Ok(SubmitResponse {
                ok: true,
                result: Some(result),
                error: None,
            })
        }
        Err(e) => Ok(SubmitResponse {
            ok: false,
            result: None,
            error: Some(e),
        }),
    }
}

/// 按开关记录本次提交使用的描述与任务，并落盘
async fn remember_last_used(state: &State<'_, AppState>, req: &SubmitRequest) {
    let snapshot = {
        let mut guard = state.config.lock().await;
        let Some(config) = guard.as_mut() else {
            return;
        };

        let task_id = req.task_ids.first().map(|s| s.as_str());
        if !config.record_last_used(&req.task_desc, task_id) {
            return;
        }

        config.clone()
    };

    let _ = crate::config::save_config_to_file(&snapshot);
}
