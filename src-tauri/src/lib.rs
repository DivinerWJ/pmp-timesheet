mod commands;
mod config;
mod pmp_api;
mod state;
mod update;

use state::AppState;
use tokio::sync::Mutex;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        // 自动更新：检查 + 下载安装（安装包需签名校验）
        .plugin(tauri_plugin_updater::Builder::new().build())
        // 更新完成后重启应用
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            // 确保配置文件存在
            let cfg_result = config::ensure_config();

            // 加载配置：缺失/为空/占位值都只是「还没配置」，交给前端走首次使用引导
            let (app_config, config_issue) = match config::load_config(None) {
                Ok(c) => (Some(c), None),
                Err(issue) => {
                    if issue.needs_setup() {
                        println!("首次使用，等待用户填写配置: {}", issue.message);
                    } else {
                        eprintln!("配置不可用: {}", issue.message);
                    }
                    (None, Some(issue))
                }
            };

            // 创建 PMP client（不自动登录，等前端调用）
            let client = app_config.as_ref().map(|cfg| pmp_api::PmpClient::new(cfg.clone()));

            let state = AppState {
                config: Mutex::new(app_config),
                client: Mutex::new(client),
                config_issue: Mutex::new(config_issue),
            };
            app.manage(state);

            if let Some(ref cfg) = cfg_result {
                if cfg.is_new || cfg.is_placeholder {
                    println!("配置文件需要填写: {}", cfg.config_path);
                }
            }

            // 开发模式下自动打开 DevTools，方便调试
            #[cfg(debug_assertions)]
            {
                if let Some(window) = app.get_webview_window("main") {
                    window.open_devtools();
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::save_preferences,
            commands::login,
            commands::login_with_config,
            commands::relogin,
            commands::list_tasks,
            commands::list_timesheets,
            commands::submit_timesheet,
            commands::toggle_devtools,
            commands::minimize_window,
            commands::close_window,
            // 多线路更新：直连 + 国内镜像
            update::mirror_nodes,
            update::update_check,
            update::update_install,
        ])
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}
