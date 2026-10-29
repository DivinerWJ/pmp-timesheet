//! 应用内更新：多线路（GitHub 直连 + 国内镜像）检查与下载。
//!
//! 官方插件只允许在编译期固定 endpoints，下载地址也固定取自 latest.json 里的 url，
//! 国内访问 GitHub 被卡住时客户端就彻底没辙。所以这里直接用插件暴露的
//! `Updater` / `Update` 自己走一遍流程：
//!
//! - 检查：tauri.conf.json 里的 GitHub 地址 + 每个镜像前缀各拼一份，按顺序尝试；
//! - 下载：按用户选择的线路顺序改写 `download_url`（镜像形如 `https://<node>/<原始地址>`），
//!   一条失败自动换下一条；包体签名校验仍由插件完成，换线路不影响校验。

use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use reqwest::Url;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tauri_plugin_updater::{Config, Updater, UpdaterExt};

/// 单个端点的检查超时：国内直连 GitHub 经常挂住，超时后立刻换下一条线路
const CHECK_TIMEOUT: Duration = Duration::from_secs(6);
/// 进度事件节流：每 ~128KB 发一次，避免被 chunk 刷爆 IPC
const PROGRESS_STEP: u64 = 128 * 1024;
/// hubp 维护的公共节点清单（纯域名数组）
const NODES_URL: &str = "https://hubp.tbedu.top/nodes.json";
/// 清单拉不到时的兜底节点
const FALLBACK_NODES: &[&str] = &["ghproxy.net", "ghfast.top", "gh-proxy.com"];
/// 节点清单缓存时长：频繁检查更新时不必每次都拉
const NODES_TTL: Duration = Duration::from_secs(600);

static NODES_CACHE: OnceLock<Mutex<Option<(Instant, Vec<String>)>>> = OnceLock::new();

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub version: String,
    pub current_version: String,
    pub body: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckOutcome {
    pub current_version: String,
    pub update: Option<UpdateInfo>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProgressEvent {
    downloaded: u64,
    total: Option<u64>,
    /// 当前实际使用的线路：空串代表 GitHub 直连，否则是镜像前缀
    source: String,
}

fn fallback_nodes() -> Vec<String> {
    FALLBACK_NODES.iter().map(|s| s.to_string()).collect()
}

/// 只接受「纯域名」形式的节点，过滤掉带协议/路径/空格的脏数据
fn is_valid_host(host: &str) -> bool {
    !host.is_empty()
        && !host.contains('/')
        && !host.contains(' ')
        && !host.contains("://")
        && host.contains('.')
}

async fn fetch_nodes() -> Vec<String> {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(6))
        .build()
    {
        Ok(client) => client,
        Err(_) => return fallback_nodes(),
    };

    let response = match client.get(NODES_URL).send().await {
        Ok(response) if response.status().is_success() => response,
        _ => return fallback_nodes(),
    };

    let body: serde_json::Value = match response.json().await {
        Ok(value) => value,
        Err(_) => return fallback_nodes(),
    };

    let hosts: Vec<String> = body
        .get("data")
        .and_then(|data| data.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str())
                .map(|host| host.trim().to_string())
                .filter(|host| is_valid_host(host))
                .collect()
        })
        .unwrap_or_default();

    if hosts.is_empty() {
        fallback_nodes()
    } else {
        hosts
    }
}

/// 节点清单：带 10 分钟缓存，拉取失败回落到内置的三个老牌节点
async fn mirror_hosts() -> Vec<String> {
    let cache = NODES_CACHE.get_or_init(|| Mutex::new(None));

    if let Ok(guard) = cache.lock() {
        if let Some((fetched_at, hosts)) = guard.as_ref() {
            if fetched_at.elapsed() < NODES_TTL && !hosts.is_empty() {
                return hosts.clone();
            }
        }
    }

    let hosts = fetch_nodes().await;
    if let Ok(mut guard) = cache.lock() {
        *guard = Some((Instant::now(), hosts.clone()));
    }
    hosts
}

/// 给前端列表用：镜像节点域名（不带协议）
#[tauri::command]
pub async fn mirror_nodes() -> Vec<String> {
    mirror_hosts().await
}

/// 读取 tauri.conf.json 里配置的官方 endpoints，作为所有线路的原始地址
fn base_endpoints(app: &AppHandle) -> Result<Vec<Url>, String> {
    let raw = app
        .config()
        .plugins
        .0
        .get("updater")
        .cloned()
        .ok_or_else(|| "缺少 updater 配置".to_string())?;
    let cfg: Config =
        serde_json::from_value(raw).map_err(|e| format!("updater 配置解析失败: {e}"))?;
    if cfg.endpoints.is_empty() {
        return Err("未配置更新地址".to_string());
    }
    Ok(cfg.endpoints)
}

/// 把镜像前缀套到原始地址上：
/// `https://ghproxy.net/` + `https://github.com/o/r/releases/...` → `https://ghproxy.net/https://github.com/...`
/// 空前缀代表直连。
fn apply_prefix(prefix: &str, base: &Url) -> Option<Url> {
    let prefix = prefix.trim();
    if prefix.is_empty() {
        return Some(base.clone());
    }
    let prefix = if prefix.ends_with('/') {
        prefix.to_string()
    } else {
        format!("{prefix}/")
    };
    Url::parse(&format!("{prefix}{base}")).ok()
}

fn build_updater(app: &AppHandle, sources: &[String]) -> Result<Updater, String> {
    let base = base_endpoints(app)?;
    let mut endpoints = base.clone();
    for prefix in sources.iter().filter(|s| !s.trim().is_empty()) {
        for url in &base {
            if let Some(proxied) = apply_prefix(prefix, url) {
                if !endpoints.contains(&proxied) {
                    endpoints.push(proxied);
                }
            }
        }
    }

    app.updater_builder()
        .endpoints(endpoints)
        .map_err(|e| e.to_string())?
        .timeout(CHECK_TIMEOUT)
        .build()
        .map_err(|e| e.to_string())
}

/// 检查更新。`sources` 是线路前缀列表（空串 = GitHub 直连），非空前缀会被用来兜底检查。
#[tauri::command]
pub async fn update_check(app: AppHandle, sources: Vec<String>) -> Result<CheckOutcome, String> {
    let updater = build_updater(&app, &sources)?;
    let found = updater.check().await.map_err(|e| e.to_string())?;
    let current_version = app.package_info().version.to_string();

    Ok(CheckOutcome {
        current_version,
        update: found.map(|u| UpdateInfo {
            version: u.version,
            current_version: u.current_version,
            body: u.body,
        }),
    })
}

/// 下载并安装更新，返回安装的版本号。
///
/// Windows 上安装程序启动后进程会退出，正常情况下这个调用不会返回。
#[tauri::command]
pub async fn update_install(app: AppHandle, sources: Vec<String>) -> Result<String, String> {
    let updater = build_updater(&app, &sources)?;
    let mut update = updater
        .check()
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "已经是最新版本，无需更新".to_string())?;

    let original = update.download_url.clone();
    let mut order = sources.clone();
    if order.is_empty() {
        order.push(String::new());
    }

    let mut last_error = String::new();
    for prefix in order {
        let Some(url) = apply_prefix(&prefix, &original) else {
            continue;
        };
        // 关键一步：把下载地址换成当前线路
        update.download_url = url;
        let label = prefix.trim().to_string();
        let handle = app.clone();
        let label_for_event = label;
        let mut downloaded: u64 = 0;
        let mut emitted: u64 = 0;

        let result = update
            .download(
                move |chunk, total| {
                    downloaded += chunk as u64;
                    if downloaded - emitted >= PROGRESS_STEP {
                        emitted = downloaded;
                        let _ = handle.emit(
                            "update-progress",
                            ProgressEvent {
                                downloaded,
                                total,
                                source: label_for_event.clone(),
                            },
                        );
                    }
                },
                || {},
            )
            .await;

        match result {
            Ok(bytes) => {
                let version = update.version.clone();
                // 走到这里说明签名已经校验通过，镜像被篡改也不会被装上
                update
                    .install(&bytes)
                    .map_err(|e| format!("安装更新失败: {e}"))?;
                return Ok(version);
            }
            Err(e) => {
                last_error = e.to_string();
            }
        }
    }

    Err(if last_error.is_empty() {
        "所有下载线路都不可用，请稍后重试".to_string()
    } else {
        last_error
    })
}
