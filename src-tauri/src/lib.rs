//! FocusMochi - AI Desktop Pet that monitors your focus
//!
//! 一个 AI 桌面宠物应用，通过摄像头监测用户专注状态，
//! 宠物会根据用户的专注程度展示不同的情绪和动画。

// 模块声明
pub mod commands;
pub mod config;
pub mod state;
pub mod storage;
pub mod vision;

use commands::AppState;
use std::sync::Arc;
use tauri::Manager;

/// 应用主入口
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("focus_mochi=debug".parse().unwrap())
        )
        .init();

    tracing::info!("FocusMochi starting...");

    // 创建应用状态（使用 Arc 包装以便在异步任务中共享）
    let app_state = Arc::new(AppState::default());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        // 注册应用状态
        .manage(app_state)
        // 注册命令处理器
        .invoke_handler(tauri::generate_handler![
            commands::get_pet_state,
            commands::start_vision,
            commands::stop_vision,
            commands::trigger_gesture,
            commands::set_demo_mood,
            commands::get_focus_stats,
            commands::reset_stats,
            commands::get_vision_status,
        ])
        .setup(|app| {
            tracing::info!("FocusMochi setup complete");

            // 获取窗口并设置透明背景
            if let Some(window) = app.get_webview_window("pet") {
                tracing::info!("Pet window found, configuring...");
                // 窗口配置已在 tauri.conf.json 中设置
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
