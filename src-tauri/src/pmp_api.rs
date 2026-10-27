use crate::config::AppConfig;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

/// PMP API 客户端
pub struct PmpClient {
    pub config: AppConfig,
    pub access_token: Option<String>,
    pub user_id: Option<String>,
    pub token_exp: Option<i64>,
    http: Client,
}

/// 登录返回信息
#[derive(Debug, Serialize)]
pub struct LoginInfo {
    pub user_id: Option<String>,
    pub expires_at: Option<String>,
    pub username: Option<String>,
}

/// 任务项
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskItem {
    #[serde(default, alias = "taskId", alias = "id")]
    pub task_id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default, alias = "projectName")]
    pub project_name: Option<String>,
    #[serde(default, alias = "projectCode")]
    pub project_code: Option<String>,
    #[serde(default, alias = "projectId")]
    pub project_id: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default, alias = "planWorkingHours")]
    pub plan_working_hours: Option<Value>,
    /// 保留原始 JSON 中的其他字段
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

/// 工时记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimesheetRecord {
    #[serde(default, alias = "submitDate")]
    pub submit_date: Option<String>,
    #[serde(default, alias = "isFill")]
    pub is_fill: Option<String>,
    #[serde(default, alias = "taskOvertime")]
    pub task_overtime: Option<Value>,
    #[serde(default, alias = "approvalStatus")]
    pub approval_status: Option<Value>,
    #[serde(default, alias = "approvalContent")]
    pub approval_content: Option<Value>,
    #[serde(default, alias = "taskType")]
    pub task_type: Option<Value>,
    #[serde(default, alias = "idList")]
    pub id_list: Option<Vec<Value>>,
    #[serde(default, alias = "officeName")]
    pub office_name: Option<String>,
    #[serde(default, alias = "createDate")]
    pub create_date: Option<String>,
    #[serde(default, alias = "isRest")]
    pub is_rest: Option<String>,
    #[serde(default, alias = "isLeave")]
    pub is_leave: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

impl PmpClient {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            access_token: None,
            user_id: None,
            token_exp: None,
            http: Client::new(),
        }
    }

    /// 检查是否仍有有效登录态（提前 60s 视为过期）
    pub fn is_logged_in(&self) -> bool {
        if self.access_token.is_none() || self.user_id.is_none() {
            return false;
        }
        match self.token_exp {
            Some(exp) => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;
                now < exp - 60
            }
            None => true,
        }
    }

    /// 无 token / 即将过期则重新登录
    pub async fn ensure_auth(&mut self) -> Result<(), String> {
        if self.is_logged_in() {
            return Ok(());
        }
        self.login().await.map(|_| ())
    }

    /// 解码 JWT payload
    fn decode_jwt_payload(token: &str) -> Result<Value, String> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() < 2 {
            return Err("无效的 JWT".to_string());
        }
        let mut payload = parts[1].replace('-', "+").replace('_', "/");
        // 补齐 Base64 padding
        while payload.len() % 4 != 0 {
            payload.push('=');
        }
        let decoded = BASE64
            .decode(&payload)
            .map_err(|e| format!("JWT Base64 解码失败: {}", e))?;
        let json: Value = serde_json::from_slice(&decoded)
            .map_err(|e| format!("JWT payload JSON 解析失败: {}", e))?;
        Ok(json)
    }

    /// 从登录响应中提取 access_token
    fn pick_token(body: &Value) -> Option<String> {
        let candidates = [
            body.get("access_token"),
            body.get("accessToken"),
            body.get("token"),
            body.get("responseBody").and_then(|r| r.get("access_token")),
            body.get("responseBody").and_then(|r| r.get("accessToken")),
            body.get("data").and_then(|d| d.get("access_token")),
            body.get("data").and_then(|d| d.get("accessToken")),
            body.get("data").and_then(|d| d.get("token")),
        ];
        for c in candidates {
            if let Some(v) = c {
                if let Some(s) = v.as_str() {
                    if !s.is_empty() {
                        return Some(s.to_string());
                    }
                }
            }
        }
        None
    }

    /// 从登录响应中提取 user_id
    fn pick_user_id(body: &Value, jwt_payload: &Value) -> Option<String> {
        let candidates = [
            jwt_payload.get("user_id"),
            jwt_payload.get("userId"),
            body.get("user_id"),
            body.get("userId"),
            body.get("responseBody").and_then(|r| r.get("user_id")),
            body.get("responseBody").and_then(|r| r.get("userId")),
            body.get("data").and_then(|d| d.get("user_id")),
            body.get("data").and_then(|d| d.get("userId")),
        ];
        for c in candidates {
            if let Some(v) = c {
                if !v.is_null() {
                    // 优先用 as_str() 避免 to_string() 给字符串加引号
                    if let Some(s) = v.as_str() {
                        return Some(s.to_string());
                    }
                    return Some(v.to_string());
                }
            }
        }
        None
    }

    /// 从业务响应中提取失败原因（HTTP 200 但业务失败的情况）
    fn pick_error_message(body: &Value) -> Option<String> {
        let obj = body.as_object()?;
        let code = obj
            .get("responseCode")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let success = obj.get("success").and_then(|v| v.as_bool());

        let is_failed = code.eq_ignore_ascii_case("failed")
            || code.eq_ignore_ascii_case("error")
            || success == Some(false);

        if !is_failed {
            return None;
        }

        let msg = obj
            .get("message")
            .or_else(|| obj.get("msg"))
            .and_then(|v| v.as_str())
            .unwrap_or("未知错误");

        Some(Self::friendly_login_error(msg))
    }

    /// 将服务端原始错误码翻译为用户可读的提示
    fn friendly_login_error(msg: &str) -> String {
        let lower = msg.to_lowercase();
        if lower.contains("bad credentials") {
            "用户名或密码错误（Bad credentials）".to_string()
        } else if lower.contains("unauthorized") || lower.contains("invalid_client") {
            "认证失败，请检查账号、密码或 Client ID/Secret".to_string()
        } else if lower.contains("account") && lower.contains("lock") {
            "账号已被锁定，请联系管理员".to_string()
        } else if lower.is_empty() {
            "登录失败：服务端未返回具体原因".to_string()
        } else {
            msg.to_string()
        }
    }

    /// 通用 HTTP 请求方法（带 401 自动重试）
    async fn request(
        &mut self,
        method: &str,
        url_path: &str,
        query: Option<Vec<(&str, String)>>,
        body: Option<&Value>,
        extra_headers: Option<Vec<(&str, &str)>>,
        skip_auth_retry: bool,
    ) -> Result<Value, String> {
        let result = self
            .request_raw(method, url_path, &query, body, &extra_headers)
            .await?;

        // 401 自动重登重试
        if !skip_auth_retry && result.0 == 401 && !url_path.contains("/auth/oauth/token") {
            self.access_token = None;
            self.token_exp = None;
            self.login().await?;
            let result2 = self
                .request_raw(method, url_path, &query, body, &extra_headers)
                .await?;
            return Ok(result2.1);
        }

        if result.0 >= 400 {
            return Err(format!("HTTP {} {} {}", result.0, method, url_path));
        }

        Ok(result.1)
    }

    /// 单次 HTTP 请求（不重试），返回 (status_code, json_body)
    async fn request_raw(
        &mut self,
        method: &str,
        url_path: &str,
        query: &Option<Vec<(&str, String)>>,
        body: Option<&Value>,
        extra_headers: &Option<Vec<(&str, &str)>>,
    ) -> Result<(u16, Value), String> {
        let full_url = if url_path.starts_with("http") {
            url_path.to_string()
        } else {
            format!("{}{}", self.config.base_url, url_path)
        };

        let url = full_url.clone();

        let mut req = match method {
            "GET" => self.http.get(&url),
            "POST" => self.http.post(&url),
            _ => return Err(format!("不支持的 HTTP 方法: {}", method)),
        };

        // 添加 query 参数
        if let Some(ref q) = query {
            req = req.query(q);
        }

        // 构建 headers
        let cookie = if let Some(ref token) = self.access_token {
            format!("language=zh; token=Bearer {}", token)
        } else {
            "language=zh".to_string()
        };

        req = req
            .header("Accept", "application/json, text/plain, */*")
            .header("Origin", &self.config.base_url)
            .header("Referer", format!("{}/workbenchPage/index", self.config.base_url))
            .header("Cookie", &cookie);

        if let Some(b) = body {
            req = req
                .header("Content-Type", "application/json;charset=UTF-8")
                .body(serde_json::to_string(b).unwrap());
        }

        if let Some(ref headers) = extra_headers {
            for (k, v) in headers {
                req = req.header(*k, *v);
            }
        }

        let res = req
            .send()
            .await
            .map_err(|e| format!("HTTP 请求失败: {}", e))?;

        let status = res.status().as_u16();
        let text = res
            .text()
            .await
            .map_err(|e| format!("读取响应失败: {}", e))?;

        let json: Value = if text.is_empty() {
            Value::Null
        } else {
            serde_json::from_str(&text).unwrap_or_else(|_| Value::Object(
                serde_json::Map::from_iter([("raw".to_string(), Value::String(text))])
            ))
        };

        Ok((status, json))
    }

    /// 登录：OAuth2 密码模式
    pub async fn login(&mut self) -> Result<LoginInfo, String> {
        let password_b64 = BASE64.encode(self.config.password.as_bytes());
        let basic = BASE64.encode(
            format!("{}:{}", self.config.client_id, self.config.client_secret).as_bytes(),
        );

        let query = vec![
            ("username", self.config.username.clone()),
            ("password", password_b64),
            ("grant_type", self.config.grant_type.clone()),
            ("scope", self.config.scope.clone()),
        ];

        let referer = format!("{}/", self.config.base_url);
        let auth_value = format!("Basic {}", basic);
        let headers: Vec<(&str, &str)> = vec![
            ("Authorization", &auth_value),
            ("isToken", "false"),
            ("Referer", &referer),
        ];

        let (status, body) = self
            .request_raw(
                "POST",
                "/auth/oauth/token",
                &Some(query),
                None,
                &Some(headers),
            )
            .await?;

        if status >= 400 {
            let detail = serde_json::to_string(&body).unwrap_or_default();
            let detail = if detail.len() > 800 {
                format!("{}...", &detail[..800])
            } else {
                detail
            };
            return Err(format!("登录失败: HTTP {}\n{}", status, detail));
        }

        // PMP 在凭证错误时仍返回 HTTP 200，失败信息在 body 里：
        // {"responseCode":"failed","message":"Bad credentials","responseBody":null,"success":false}
        if let Some(msg) = Self::pick_error_message(&body) {
            return Err(format!("登录失败: {}", msg));
        }

        let token = match Self::pick_token(&body) {
            Some(t) => t,
            None => {
                let detail = serde_json::to_string(&body).unwrap_or_default();
                return Err(format!(
                    "登录成功但未找到 access_token，请检查响应结构\n{}",
                    detail
                ));
            }
        };

        let payload = Self::decode_jwt_payload(&token)?;
        let user_id = Self::pick_user_id(&body, &payload);
        let token_exp = payload.get("exp").and_then(|v| v.as_i64());

        self.access_token = Some(token.clone());
        self.user_id = user_id.clone();
        self.token_exp = token_exp;

        let expires_at = token_exp.map(|exp| {
            let secs = exp as u64;
            let dt = std::time::UNIX_EPOCH + std::time::Duration::from_secs(secs);
            format!("{:?}", dt)
        });

        let username = payload
            .get("user_name")
            .or_else(|| payload.get("username"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(LoginInfo {
            user_id,
            expires_at,
            username,
        })
    }

    /// 查询进行中的叶子任务
    pub async fn list_doing_tasks(
        &mut self,
        page_no: u32,
        page_size: u32,
    ) -> Result<Vec<TaskItem>, String> {
        self.ensure_auth().await?;

        let auth_header = self.auth_header()?;
        let body = serde_json::json!({
            "checkType": 2,
            "isLeafNode": "1",
            "pageNo": page_no,
            "pageSize": page_size,
            "status": null,
            "statusList": ["0", "3", "4", "5"]
        });

        let path = format!(
            "/pm/projectTask/selectDoingProjectTaskList/{}/{}",
            page_no, page_size
        );

        let response = self
            .request(
                "POST",
                &path,
                None,
                Some(&body),
                Some(vec![("Authorization", &auth_header)]),
                false,
            )
            .await?;

        Ok(self::unwrap_list(&response))
    }

    /// 查询工时记录
    pub async fn list_timesheets(
        &mut self,
        page_no: u32,
        page_size: u32,
    ) -> Result<Vec<TimesheetRecord>, String> {
        self.ensure_auth().await?;

        let auth_header = self.auth_header()?;
        let body = serde_json::json!({
            "pageNo": page_no,
            "pageSize": page_size
        });

        let path = format!(
            "/independence/timesheetNew/selectAllByTaskDesc/{}/{}",
            page_no, page_size
        );

        let response = self
            .request(
                "POST",
                &path,
                None,
                Some(&body),
                Some(vec![("Authorization", &auth_header)]),
                false,
            )
            .await?;

        Ok(self::unwrap_list(&response))
    }

    /// 提交工时
    pub async fn submit_timesheet(
        &mut self,
        day: &str,
        task_ids: &[String],
        task_desc: &str,
        hours: f64,
        save_or_submit: Option<String>,
    ) -> Result<Value, String> {
        self.ensure_auth().await?;

        if day.is_empty() {
            return Err("缺少 day (yyyy-MM-dd)".to_string());
        }
        if task_ids.is_empty() {
            return Err("tasks 不能为空".to_string());
        }
        let user_id = self
            .user_id
            .clone()
            .ok_or_else(|| "缺少 userId，请先 login".to_string())?;
        if task_desc.is_empty() {
            return Err("缺少 taskDesc（工作描述）".to_string());
        }

        let sos = save_or_submit.unwrap_or_else(|| self.config.save_or_submit.clone());

        // 先获取任务详情
        let tasks = self.list_doing_tasks(1, 100).await?;
        let selected: Vec<&TaskItem> = tasks
            .iter()
            .filter(|t| {
                let tid = t
                    .task_id
                    .as_deref()
                    .or_else(|| t.extra.get("id").and_then(|v| v.as_str()));
                tid.map(|id| task_ids.contains(&id.to_string()))
                    .unwrap_or(false)
            })
            .collect();

        if selected.is_empty() {
            return Err("未找到选中的任务".to_string());
        }

        let task_list: Vec<Value> = selected
            .iter()
            .map(|t| {
                let mut item = serde_json::to_value(t).unwrap_or(Value::Null);
                if let Some(obj) = item.as_object_mut() {
                    obj.insert("taskDesc".to_string(), Value::String(task_desc.to_string()));
                    obj.insert(
                        "taskOvertime".to_string(),
                        Value::Number(serde_json::Number::from_f64(hours).unwrap_or(0.into())),
                    );
                    obj.insert("taskType".to_string(), serde_json::json!(1));
                }
                item
            })
            .collect();

        let path = format!("/independence/timesheetNew/saveOrUpdate/{}", day);
        let body = serde_json::json!({
            "fiileadUserId": user_id,
            "submitDate": day,
            "day": day,
            "taskList": task_list,
            "idList": [],
            "detailIdList": [],
            "saveOrSubmit": sos
        });

        let auth_header = self.auth_header()?;
        self.request(
            "POST",
            &path,
            None,
            Some(&body),
            Some(vec![("Authorization", &auth_header)]),
            false,
        )
        .await
    }

    fn auth_header(&self) -> Result<String, String> {
        match &self.access_token {
            Some(token) => Ok(format!("Bearer {}", token)),
            None => Err("尚未登录，请先调用 login()".to_string()),
        }
    }
}

/// 从 API 响应中提取列表数组
fn unwrap_list<T: for<'de> Deserialize<'de>>(body: &Value) -> Vec<T> {
    // 直接是数组
    if let Some(arr) = body.as_array() {
        return arr
            .iter()
            .filter_map(|v| serde_json::from_value(v.clone()).ok())
            .collect();
    }

    // 尝试各种嵌套路径
    let candidates = [
        body.get("responseBody")
            .and_then(|r| r.get("records"))
            .or_else(|| body.get("responseBody").and_then(|r| r.get("list")))
            .or_else(|| body.get("responseBody").and_then(|r| r.get("rows")))
            .or_else(|| body.get("responseBody").and_then(|r| r.get("data")))
            .or_else(|| body.get("responseBody")),
        body.get("data")
            .and_then(|d| d.get("records"))
            .or_else(|| body.get("data").and_then(|d| d.get("list")))
            .or_else(|| body.get("data").and_then(|d| d.get("rows")))
            .or_else(|| body.get("data").and_then(|d| d.get("data"))),
        body.get("records"),
        body.get("list"),
        body.get("rows"),
    ];

    for c in candidates {
        if let Some(arr) = c.and_then(|v| v.as_array()) {
            let items: Vec<T> = arr
                .iter()
                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                .collect();
            if !items.is_empty() {
                return items;
            }
        }
    }

    Vec::new()
}

/// 获取今天的日期字符串 yyyy-MM-dd
pub fn today_str() -> String {
    let now = std::time::SystemTime::now();
    let duration = now.duration_since(UNIX_EPOCH).unwrap();
    let secs = duration.as_secs() as i64;

    // 简单日期计算（使用 UTC+8）
    let secs_tz = secs + 8 * 3600;
    let days = secs_tz / 86400;

    // 从 1970-01-01 开始计算
    let (y, m, d) = days_to_ymd(days);
    format!("{:04}-{:02}-{:02}", y, m, d)
}

fn days_to_ymd(mut days: i64) -> (i64, i64, i64) {
    // 算法：从 1970-01-01 的天数转换为年月日
    let mut y = 1970;
    loop {
        let days_in_year = if is_leap_year(y) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        y += 1;
    }

    let leap = is_leap_year(y);
    let month_days = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut m = 0;
    for (i, &md) in month_days.iter().enumerate() {
        if days < md {
            m = i as i64 + 1;
            break;
        }
        days -= md;
    }
    (y, m, days + 1)
}

fn is_leap_year(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_jwt_payload() {
        // 一个简单的 JWT-like token（仅用于测试解码）
        let header = BASE64.encode(r#"{"alg":"HS256"}"#);
        let payload = BASE64.encode(r#"{"user_id":"123","exp":9999999999}"#);
        let token = format!("{}.{}.signature", header, payload);

        let result = PmpClient::decode_jwt_payload(&token).unwrap();
        assert_eq!(result["user_id"], "123");
        assert_eq!(result["exp"], 9999999999_i64);
    }

    #[test]
    fn test_pick_token() {
        let body = serde_json::json!({"access_token": "test_token_123"});
        assert_eq!(
            PmpClient::pick_token(&body),
            Some("test_token_123".to_string())
        );

        let body2 = serde_json::json!({"data": {"token": "nested_token"}});
        assert_eq!(
            PmpClient::pick_token(&body2),
            Some("nested_token".to_string())
        );

        let body3 = serde_json::json!({"foo": "bar"});
        assert_eq!(PmpClient::pick_token(&body3), None);
    }

    #[test]
    fn test_unwrap_list() {
        let body = serde_json::json!({"responseBody": {"records": [{"name": "task1"}, {"name": "task2"}]}});
        let list: Vec<TaskItem> = unwrap_list(&body);
        assert_eq!(list.len(), 2);

        let body2 = serde_json::json!([{"name": "direct1"}]);
        let list2: Vec<TaskItem> = unwrap_list(&body2);
        assert_eq!(list2.len(), 1);
    }

    #[test]
    fn test_today_str() {
        let today = today_str();
        // 格式检查：yyyy-MM-dd
        assert_eq!(today.len(), 10);
        assert_eq!(&today[4..5], "-");
        assert_eq!(&today[7..8], "-");
    }
}
