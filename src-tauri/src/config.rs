use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// 应用配置结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_base_url", alias = "baseUrl")]
    pub base_url: String,
    pub username: String,
    pub password: String,
    #[serde(alias = "clientId")]
    pub client_id: String,
    #[serde(alias = "clientSecret")]
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
    /// 运行时填充，不参与序列化
    #[serde(skip)]
    pub config_path: String,
}

fn default_base_url() -> String {
    "http://localhost:8080".to_string()
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

const _REQUIRED_FIELDS: &[&str] = &["username", "password", "client_id", "client_secret"];

const EXAMPLE_CONFIG: &str = r#"{
  "base_url": "http://localhost:8080",
  "username": "your_username",
  "password": "your_plaintext_password",
  "client_id": "your_client_id",
  "client_secret": "your_client_secret",
  "grant_type": "password",
  "scope": "server",
  "default_hours": 7.5,
  "save_or_submit": "2",
  "default_task_desc": ""
}
"#;

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
    let is_placeholder = content.contains("your_plaintext_password") || content.contains("your_username");

    Some(ConfigState {
        config_path: path_str,
        is_new: false,
        is_placeholder,
    })
}

/// 加载配置文件
pub fn load_config(config_path: Option<String>) -> Result<AppConfig, String> {
    let path = match config_path {
        Some(p) => PathBuf::from(p),
        None => default_config_path(),
    };

    if !path.exists() {
        return Err(format!(
            "缺少配置文件: {}\n请编辑配置文件并填写账号信息",
            path.display()
        ));
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;

    let mut config: AppConfig = serde_json::from_str(&content)
        .map_err(|e| format!("配置文件格式错误: {}", e))?;

    // 校验必填项
    let mut missing = Vec::new();
    if config.username.is_empty() || config.username == "your_username" {
        missing.push("username");
    }
    if config.password.is_empty() || config.password == "your_plaintext_password" {
        missing.push("password");
    }
    if config.client_id.is_empty() || config.client_id == "your_client_id" {
        missing.push("client_id");
    }
    if config.client_secret.is_empty() || config.client_secret == "your_client_secret" {
        missing.push("client_secret");
    }

    if !missing.is_empty() {
        return Err(format!("config.json 缺少必填项或仍为占位值: {}", missing.join(", ")));
    }

    // 去除 base_url 末尾斜杠
    config.base_url = config.base_url.trim_end_matches('/').to_string();
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
        "defaultTaskDesc": config.default_task_desc
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

    #[test]
    fn test_load_valid_config() {
        let content = r#"{
            "base_url": "http://localhost:8080",
            "username": "testuser",
            "password": "testpass",
            "client_id": "test-client",
            "client_secret": "test-secret"
        }"#;
        let f = write_temp_config(content);
        let config = load_config(Some(f.path().to_string_lossy().to_string()));
        assert!(config.is_ok());
        let c = config.unwrap();
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
        let f = write_temp_config(content);
        let result = load_config(Some(f.path().to_string_lossy().to_string()));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("username"));
    }

    #[test]
    fn test_load_placeholder_config() {
        let content = r#"{
            "username": "your_username",
            "password": "your_plaintext_password",
            "client_id": "your_client_id",
            "client_secret": "your_client_secret"
        }"#;
        let f = write_temp_config(content);
        let result = load_config(Some(f.path().to_string_lossy().to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_default_values() {
        let content = r#"{
            "username": "testuser",
            "password": "testpass",
            "client_id": "test-client",
            "client_secret": "test-secret"
        }"#;
        let f = write_temp_config(content);
        let config = load_config(Some(f.path().to_string_lossy().to_string())).unwrap();
        assert_eq!(config.base_url, "http://localhost:8080");
        assert_eq!(config.grant_type, "password");
        assert_eq!(config.scope, "server");
        assert_eq!(config.default_hours, 7.5);
        assert_eq!(config.save_or_submit, "2");
    }
}
