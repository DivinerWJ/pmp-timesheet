use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

/// 示例配置里的占位值，读到这些值视为“没填”
const PLACEHOLDER_USERNAME: &str = "your_username";
const PLACEHOLDER_PASSWORD: &str = "your_plaintext_password";
const PLACEHOLDER_CLIENT_ID: &str = "your_client_id";
const PLACEHOLDER_CLIENT_SECRET: &str = "your_client_secret";

/// 应用配置结构体
///
/// 所有字段都不设业务默认值：没填就是空，由登录页让用户显式填写，
/// 避免出现「莫名其妙指向 localhost」这类误导。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default, alias = "baseUrl")]
    pub base_url: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default, alias = "clientId")]
    pub client_id: String,
    #[serde(default, alias = "clientSecret")]
    pub client_secret: String,
    #[serde(default = "default_grant_type", alias = "grantType")]
    pub grant_type: String,
    #[serde(default = "default_scope")]
    pub scope: String,
    #[serde(default = "default_hours", alias = "defaultHours")]
    pub default_hours: f64,
    #[serde(default = "default_save_or_submit", alias = "saveOrSubmit")]
    pub save_or_submit: String,
    #[serde(default, alias = "defaultTaskDesc")]
    pub default_task_desc: String,
    /// 记忆开关：提交后保留工作描述，下次打开自动填入
    #[serde(default = "default_true", alias = "rememberTaskDesc")]
    pub remember_task_desc: bool,
    /// 记忆开关：记住选中的任务，下次打开自动选中
    #[serde(default = "default_true", alias = "rememberTaskId")]
    pub remember_task_id: bool,
    /// 上次提交使用的工作描述（仅 remember_task_desc 为真时有意义）
    #[serde(default, alias = "lastTaskDesc")]
    pub last_task_desc: String,
    /// 上次提交选中的任务 ID（仅 remember_task_id 为真时有意义）
    #[serde(default, alias = "lastTaskId")]
    pub last_task_id: String,
    /// 运行时填充，不参与序列化
    #[serde(skip)]
    pub config_path: String,
}

fn default_grant_type() -> String {
    "password".to_string()
}
fn default_scope() -> String {
    "server".to_string()
}
fn default_hours() -> f64 {
    7.5
}
fn default_save_or_submit() -> String {
    "2".to_string()
}
/// 记忆类开关默认开启：装了就用，想清空自己去设置里关
pub fn default_true() -> bool {
    true
}

const EXAMPLE_CONFIG: &str = r#"{
  "base_url": "",
  "username": "",
  "password": "",
  "client_id": "",
  "client_secret": "",
  "grant_type": "password",
  "scope": "server",
  "default_hours": 7.5,
  "save_or_submit": "2",
  "default_task_desc": "",
  "remember_task_desc": true,
  "remember_task_id": true,
  "last_task_desc": "",
  "last_task_id": ""
}
"#;

/// 配置不可用的原因分类
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigIssueKind {
    /// 首次使用 / 配置不完整：属于正常状态，引导用户填写即可，不要当成报错
    SetupRequired,
    /// 配置文件存在但读不了、或内容不是合法 JSON：需要提示用户
    Invalid,
}

/// 配置不可用的原因（message 面向用户，可直接展示）
#[derive(Debug, Clone)]
pub struct ConfigIssue {
    pub kind: ConfigIssueKind,
    pub message: String,
}

impl ConfigIssue {
    fn setup_required(message: String) -> Self {
        Self {
            kind: ConfigIssueKind::SetupRequired,
            message,
        }
    }

    fn invalid(message: String) -> Self {
        Self {
            kind: ConfigIssueKind::Invalid,
            message,
        }
    }

    /// 是否只是「还没配置好」，前端据此展示首次使用引导而不是红色错误
    pub fn needs_setup(&self) -> bool {
        self.kind == ConfigIssueKind::SetupRequired
    }
}

/// 获取平台标准配置路径
pub fn config_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("PMP_TIMESHEET_CONFIG_DIR") {
        return PathBuf::from(dir);
    }

    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

    if cfg!(target_os = "macos") {
        home.join("Library").join("Application Support").join("PMPTimesheet")
    } else if cfg!(target_os = "windows") {
        let appdata = std::env::var("APPDATA")
            .unwrap_or_else(|_| home.join("AppData").join("Roaming").to_string_lossy().to_string());
        PathBuf::from(appdata).join("PMPTimesheet")
    } else {
        home.join(".config").join("pmp-timesheet")
    }
}

/// 获取默认配置文件路径
pub fn default_config_path() -> PathBuf {
    config_dir().join("config.json")
}

/// 配置状态信息
pub struct ConfigState {
    pub config_path: String,
    pub is_new: bool,
    pub is_placeholder: bool,
}

/// 确保配置文件存在，首次启动时创建示例配置
pub fn ensure_config() -> Option<ConfigState> {
    let path = default_config_path();
    let path_str = path.to_string_lossy().to_string();

    if !path.exists() {
        // 创建目录
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        // 写入示例配置
        if let Err(e) = fs::write(&path, EXAMPLE_CONFIG) {
            eprintln!("无法创建示例配置文件: {}", e);
            return None;
        }
        println!("已创建示例配置文件: {}", path_str);
        return Some(ConfigState {
            config_path: path_str,
            is_new: true,
            is_placeholder: true,
        });
    }

    // 检查是否为占位内容
    let content = fs::read_to_string(&path).unwrap_or_default();
    let is_placeholder =
        content.contains(PLACEHOLDER_PASSWORD) || content.contains(PLACEHOLDER_USERNAME);

    Some(ConfigState {
        config_path: path_str,
        is_new: false,
        is_placeholder,
    })
}

/// 加载配置文件
///
/// 注意：「没有配置 / 配置为空 / 还是占位值」都返回 [`ConfigIssueKind::SetupRequired`]，
/// 调用方应当把它当作首次使用的正常状态处理，而不是展示成报错。
pub fn load_config(config_path: Option<String>) -> Result<AppConfig, ConfigIssue> {
    let path = match config_path {
        Some(p) => PathBuf::from(p),
        None => default_config_path(),
    };
    let path_display = path.display();

    if !path.exists() {
        return Err(ConfigIssue::setup_required(format!(
            "首次使用：还没有账号配置（{}）。请在下方填写服务器地址、用户名和密码，登录成功后会自动保存。",
            path_display
        )));
    }

    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            return Err(ConfigIssue::invalid(format!(
                "无法读取配置文件 {}（{}）。可以直接在下方重新登录，成功后会覆盖该文件。",
                path_display, e
            )))
        }
    };

    // 空文件（0 字节、只有空白）属于“还没配置”，不要报 JSON 解析错误
    if content.trim().is_empty() {
        return Err(ConfigIssue::setup_required(format!(
            "首次使用：配置文件 {} 还是空的。请在下方填写账号信息后登录。",
            path_display
        )));
    }

    let raw: Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            return Err(ConfigIssue::invalid(format!(
                "配置文件 {} 不是合法的 JSON（{}）。修正后重启，或直接在下方重新登录覆盖它。",
                path_display, e
            )))
        }
    };

    let mut obj = match raw {
        Value::Object(map) => map,
        // 内容为 null 等价于没配置
        Value::Null => {
            return Err(ConfigIssue::setup_required(format!(
                "首次使用：配置文件 {} 还是空的。请在下方填写账号信息后登录。",
                path_display
            )))
        }
        _ => {
            return Err(ConfigIssue::invalid(format!(
                "配置文件 {} 的内容不是 JSON 对象，请修正后重启，或直接在下方重新登录覆盖它。",
                path_display
            )))
        }
    };

    // null 字段等价于「没填」，直接丢掉，避免整份配置因类型不匹配而无法使用
    obj.retain(|_, v| !v.is_null());

    let mut config: AppConfig = match serde_json::from_value(Value::Object(obj)) {
        Ok(c) => c,
        Err(e) => {
            return Err(ConfigIssue::invalid(format!(
                "配置文件 {} 的字段类型有误（{}）。修正后重启，或直接在下方重新登录覆盖它。",
                path_display, e
            )))
        }
    };

    // 占位值等同于没填
    if config.username.trim() == PLACEHOLDER_USERNAME {
        config.username = String::new();
    }
    if config.password == PLACEHOLDER_PASSWORD {
        config.password = String::new();
    }
    if config.client_id.trim() == PLACEHOLDER_CLIENT_ID {
        config.client_id = String::new();
    }
    if config.client_secret == PLACEHOLDER_CLIENT_SECRET {
        config.client_secret = String::new();
    }

    // 服务器地址、账号密码、Client 凭据都是必填项
    let mut missing: Vec<&str> = Vec::new();
    if config.base_url.trim().is_empty() {
        missing.push("base_url");
    }
    if config.username.trim().is_empty() {
        missing.push("username");
    }
    if config.password.is_empty() {
        missing.push("password");
    }
    if config.client_id.trim().is_empty() {
        missing.push("client_id");
    }
    if config.client_secret.is_empty() {
        missing.push("client_secret");
    }

    if !missing.is_empty() {
        return Err(ConfigIssue::setup_required(format!(
            "首次使用：配置里还没有填写 {}（{}）。请在下方填写账号信息，登录成功后会自动保存。",
            missing.join("、"),
            path_display
        )));
    }

    // 归一化可选配置，避免脏值影响使用
    config.base_url = config.base_url.trim().trim_end_matches('/').to_string();
    if !config.default_hours.is_finite()
        || config.default_hours <= 0.0
        || config.default_hours > 24.0
    {
        config.default_hours = default_hours();
    }
    if config.save_or_submit.trim().is_empty() {
        config.save_or_submit = default_save_or_submit();
    }

    config.config_path = path.to_string_lossy().to_string();

    Ok(config)
}

/// 前端展示用的配置摘要（不含密码）
#[derive(Serialize)]
#[allow(dead_code)]
pub struct ConfigSummary {
    pub username: String,
    pub default_hours: f64,
    pub save_or_submit: String,
    pub config_path: String,
    pub config_error: Option<String>,
}

impl AppConfig {
    /// 未配置时的空配置：字段全为空，仅提供配置路径，不可用于登录
    pub fn empty() -> Self {
        AppConfig {
            base_url: String::new(),
            username: String::new(),
            password: String::new(),
            client_id: String::new(),
            client_secret: String::new(),
            grant_type: default_grant_type(),
            scope: default_scope(),
            default_hours: default_hours(),
            save_or_submit: default_save_or_submit(),
            default_task_desc: String::new(),
            remember_task_desc: default_true(),
            remember_task_id: default_true(),
            last_task_desc: String::new(),
            last_task_id: String::new(),
            config_path: default_config_path().to_string_lossy().to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn to_summary(&self) -> ConfigSummary {
        ConfigSummary {
            username: self.username.clone(),
            default_hours: self.default_hours,
            save_or_submit: self.save_or_submit.clone(),
            config_path: self.config_path.clone(),
            config_error: None,
        }
    }

    /// 切换「记住工作描述 / 记住选中的任务」开关。
    ///
    /// 关闭时顺手清掉已记住的内容，避免下次重新打开开关又冒出旧值。
    /// 返回配置是否发生变化（没变化就不必写盘）。
    pub fn set_remember_flags(
        &mut self,
        remember_desc: Option<bool>,
        remember_id: Option<bool>,
    ) -> bool {
        let mut changed = false;

        if let Some(enabled) = remember_desc {
            if self.remember_task_desc != enabled {
                self.remember_task_desc = enabled;
                changed = true;
            }
            if !enabled && !self.last_task_desc.is_empty() {
                self.last_task_desc.clear();
                changed = true;
            }
        }

        if let Some(enabled) = remember_id {
            if self.remember_task_id != enabled {
                self.remember_task_id = enabled;
                changed = true;
            }
            if !enabled && !self.last_task_id.is_empty() {
                self.last_task_id.clear();
                changed = true;
            }
        }

        changed
    }

    /// 提交成功后按开关记住本次使用的描述与任务。
    ///
    /// 开关关掉时完全不写入，保证「不保存」是真的不保存。
    /// 返回配置是否发生变化。
    pub fn record_last_used(&mut self, task_desc: &str, task_id: Option<&str>) -> bool {
        let mut changed = false;

        if self.remember_task_desc {
            let desc = task_desc.trim();
            if !desc.is_empty() && self.last_task_desc != desc {
                self.last_task_desc = desc.to_string();
                changed = true;
            }
        }

        if self.remember_task_id {
            if let Some(id) = task_id.map(str::trim).filter(|s| !s.is_empty()) {
                if self.last_task_id != id {
                    self.last_task_id = id.to_string();
                    changed = true;
                }
            }
        }

        changed
    }
}

/// 保存配置到文件
pub fn save_config_to_file(config: &AppConfig) -> Result<(), String> {
    let path = default_config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败: {}", e))?;
    }

    let json = serde_json::json!({
        "baseUrl": config.base_url,
        "username": config.username,
        "password": config.password,
        "clientId": config.client_id,
        "clientSecret": config.client_secret,
        "grantType": config.grant_type,
        "scope": config.scope,
        "defaultHours": config.default_hours,
        "saveOrSubmit": config.save_or_submit,
        "defaultTaskDesc": config.default_task_desc,
        "rememberTaskDesc": config.remember_task_desc,
        "rememberTaskId": config.remember_task_id,
        "lastTaskDesc": config.last_task_desc,
        "lastTaskId": config.last_task_id
    });

    let content = serde_json::to_string_pretty(&json)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    fs::write(&path, content).map_err(|e| format!("写入配置文件失败: {}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_temp_config(content: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f
    }

    fn load(content: &str) -> Result<AppConfig, ConfigIssue> {
        let f = write_temp_config(content);
        load_config(Some(f.path().to_string_lossy().to_string()))
    }

    #[test]
    fn test_load_valid_config() {
        let content = r#"{
            "base_url": "http://localhost:8080",
            "username": "testuser",
            "password": "testpass",
            "client_id": "test-client",
            "client_secret": "test-secret"
        }"#;
        let c = load(content).unwrap();
        assert_eq!(c.username, "testuser");
        assert_eq!(c.default_hours, 7.5);
    }

    #[test]
    fn test_load_missing_required_fields() {
        let content = r#"{
            "username": "",
            "password": "testpass",
            "client_id": "test-client",
            "client_secret": "test-secret"
        }"#;
        let err = load(content).unwrap_err();
        assert!(err.message.contains("username"));
        assert_eq!(err.kind, ConfigIssueKind::SetupRequired);
        assert!(err.needs_setup());
    }

    #[test]
    fn test_load_placeholder_config() {
        let content = r#"{
            "username": "your_username",
            "password": "your_plaintext_password",
            "client_id": "your_client_id",
            "client_secret": "your_client_secret"
        }"#;
        let err = load(content).unwrap_err();
        assert_eq!(err.kind, ConfigIssueKind::SetupRequired);
    }

    #[test]
    fn test_empty_file_is_setup_required() {
        let err = load("").unwrap_err();
        assert_eq!(err.kind, ConfigIssueKind::SetupRequired);
        assert!(err.needs_setup());
    }

    #[test]
    fn test_empty_object_is_setup_required() {
        let err = load("{}").unwrap_err();
        assert_eq!(err.kind, ConfigIssueKind::SetupRequired);
    }

    #[test]
    fn test_null_fields_are_ignored() {
        let content = r#"{
            "base_url": "http://pmp.example.com/",
            "username": "testuser",
            "password": "testpass",
            "client_id": "test-client",
            "client_secret": "test-secret",
            "default_hours": null,
            "save_or_submit": null
        }"#;
        let c = load(content).unwrap();
        assert_eq!(c.username, "testuser");
        // 末尾斜杠会被归一化掉
        assert_eq!(c.base_url, "http://pmp.example.com");
        assert_eq!(c.default_hours, 7.5);
        assert_eq!(c.save_or_submit, "2");
    }

    #[test]
    fn test_missing_base_url_is_setup_required() {
        let content = r#"{
            "username": "testuser",
            "password": "testpass"
        }"#;
        let err = load(content).unwrap_err();
        assert_eq!(err.kind, ConfigIssueKind::SetupRequired);
        assert!(err.message.contains("base_url"));
    }

    #[test]
    fn test_broken_json_is_invalid() {
        let err = load("{ this is not json ").unwrap_err();
        assert_eq!(err.kind, ConfigIssueKind::Invalid);
        assert!(!err.needs_setup());
    }

    #[test]
    fn test_missing_config_file_is_setup_required() {
        let err = load_config(Some("/nonexistent/path/config.json".to_string())).unwrap_err();
        assert_eq!(err.kind, ConfigIssueKind::SetupRequired);
    }

    #[test]
    fn test_client_credentials_required() {
        let content = r#"{
            "base_url": "http://pmp.example.com",
            "username": "testuser",
            "password": "testpass"
        }"#;
        let err = load(content).unwrap_err();
        assert_eq!(err.kind, ConfigIssueKind::SetupRequired);
        assert!(err.message.contains("client_id"));
        assert!(err.message.contains("client_secret"));
    }

    #[test]
    fn test_default_values() {
        let content = r#"{
            "base_url": "http://pmp.example.com",
            "username": "testuser",
            "password": "testpass",
            "client_id": "test-client",
            "client_secret": "test-secret"
        }"#;
        let config = load(content).unwrap();
        assert_eq!(config.base_url, "http://pmp.example.com");
        assert_eq!(config.grant_type, "password");
        assert_eq!(config.scope, "server");
        assert_eq!(config.default_hours, 7.5);
        assert_eq!(config.save_or_submit, "2");
    }

    #[test]
    fn test_remember_flags_default_on_for_old_config() {
        // 老配置文件里没有这两个开关：默认开启，且没有「上次使用」的残留
        let content = r#"{
            "base_url": "http://pmp.example.com",
            "username": "testuser",
            "password": "testpass",
            "client_id": "test-client",
            "client_secret": "test-secret"
        }"#;
        let config = load(content).unwrap();
        assert!(config.remember_task_desc);
        assert!(config.remember_task_id);
        assert!(config.last_task_desc.is_empty());
        assert!(config.last_task_id.is_empty());
    }

    #[test]
    fn test_remember_fields_are_read_from_camel_case() {
        let content = r#"{
            "base_url": "http://pmp.example.com",
            "username": "testuser",
            "password": "testpass",
            "client_id": "test-client",
            "client_secret": "test-secret",
            "rememberTaskDesc": false,
            "rememberTaskId": false,
            "lastTaskDesc": "写文档",
            "lastTaskId": "T-1001"
        }"#;
        let config = load(content).unwrap();
        assert!(!config.remember_task_desc);
        assert!(!config.remember_task_id);
        assert_eq!(config.last_task_desc, "写文档");
        assert_eq!(config.last_task_id, "T-1001");
    }

    fn config_with_remember(desc: bool, task: bool) -> AppConfig {
        let mut config = AppConfig::empty();
        config.remember_task_desc = desc;
        config.remember_task_id = task;
        config
    }

    #[test]
    fn test_record_last_used_when_enabled() {
        let mut config = config_with_remember(true, true);
        assert!(config.record_last_used("  写周报  ", Some("T-1")));
        // 描述会去掉首尾空白
        assert_eq!(config.last_task_desc, "写周报");
        assert_eq!(config.last_task_id, "T-1");
        // 同样的内容再提交一次不算变化，避免重复写盘
        assert!(!config.record_last_used("写周报", Some("T-1")));
    }

    #[test]
    fn test_record_last_used_when_disabled() {
        let mut config = config_with_remember(false, false);
        assert!(!config.record_last_used("写周报", Some("T-1")));
        assert!(config.last_task_desc.is_empty());
        assert!(config.last_task_id.is_empty());
    }

    #[test]
    fn test_record_last_used_only_one_enabled() {
        let mut config = config_with_remember(true, false);
        assert!(config.record_last_used("只记描述", Some("T-9")));
        assert_eq!(config.last_task_desc, "只记描述");
        // 任务开关关着，任务不该被记住
        assert!(config.last_task_id.is_empty());
    }

    #[test]
    fn test_toggle_off_clears_remembered_values() {
        let mut config = config_with_remember(true, true);
        config.last_task_desc = "旧描述".to_string();
        config.last_task_id = "T-1".to_string();

        assert!(config.set_remember_flags(Some(false), None));
        assert!(!config.remember_task_desc);
        // 关掉「记住工作描述」应清掉已存的内容
        assert!(config.last_task_desc.is_empty());
        // 任务开关没动，值保留
        assert!(config.remember_task_id);
        assert_eq!(config.last_task_id, "T-1");
    }

    #[test]
    fn test_toggle_without_change_does_not_touch_config() {
        let mut config = config_with_remember(true, true);
        // 已经开启，再传 true 不算变化
        assert!(!config.set_remember_flags(Some(true), Some(true)));
        assert!(!config.set_remember_flags(None, None));
    }
}
