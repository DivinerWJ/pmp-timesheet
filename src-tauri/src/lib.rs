mod commands;
mod config;
mod pmp_api;
mod state;

use state::AppState;
use tokio::sync::Mutex;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // 确保配置文件存在
            let cfg_result = config::ensure_config();

            // 加载配置
            let app_config = match config::load_config(None) {
                Ok(c) => Some(c),
                Err(e) => {
                    eprintln!("配置加载失败: {}", e);
                    let state = AppState {
                        config: Mutex::new(None),
                        client: Mutex::new(None),
                        config_error: Mutex::new(Some(e.to_string())),
                    };
                    app.manage(state);
                    return Ok(());
                }
            };

            // 创建 PMP client（不自动登录，等前端调用）
            let client = app_config.as_ref().map(|cfg| pmp_api::PmpClient::new(cfg.clone()));

            let state = AppState {
                config: Mutex::new(app_config),
                client: Mutex::new(client),
                config_error: Mutex::new(None),
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
            commands::login,
            commands::login_with_config,
            commands::relogin,
            commands::list_tasks,
            commands::list_timesheets,
            commands::submit_timesheet,
            commands::toggle_devtools,
            commands::minimize_window,
            commands::close_window,
        ])
        .run(tauri::generate_context!())
        .expect("启动 Tauri 应用失败");
}
