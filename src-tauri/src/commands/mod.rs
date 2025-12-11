//! Tauri 命令模块
//!
//! 定义前端可调用的 Tauri 命令
//! 包括宠物状态管理、视觉检测控制等功能

use crate::state::{FocusStats, GestureType, PetMood, PetStateMachine, PetStateConfig};
use crate::vision::{FocusState, VisionProcessor, VisionProcessorConfig};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use parking_lot::Mutex;
use tokio::sync::watch;

/// 应用全局状态
pub struct AppState {
    /// 宠物状态机
    pub pet_state_machine: Mutex<PetStateMachine>,
    /// 专注统计
    pub focus_stats: Mutex<FocusStats>,
    /// 视觉处理器
    pub vision_processor: Mutex<Option<Arc<VisionProcessor>>>,
    /// 专注状态接收器
    pub focus_state_rx: Mutex<Option<watch::Receiver<FocusState>>>,
    /// 是否正在运行视觉检测
    pub vision_running: Mutex<bool>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            pet_state_machine: Mutex::new(PetStateMachine::new(PetStateConfig::default())),
            focus_stats: Mutex::new(FocusStats {
                total_focus_ms: 0,
                current_mood: PetMood::Idle,
                focus_level: crate::state::FocusLevel::Away,
                focus_score: 0.0,
            }),
            vision_processor: Mutex::new(None),
            focus_state_rx: Mutex::new(None),
            vision_running: Mutex::new(false),
        }
    }
}

/// 获取当前宠物状态
#[tauri::command]
pub fn get_pet_state(state: State<'_, AppState>) -> PetStateResponse {
    let machine = state.pet_state_machine.lock();
    let stats = state.focus_stats.lock().clone();
    let vision_running = *state.vision_running.lock();

    // 如果视觉检测正在运行，尝试获取最新的专注状态
    let (focus_score, face_detected) = if vision_running {
        if let Some(ref rx) = *state.focus_state_rx.lock() {
            let focus_state = rx.borrow().clone();
            (focus_state.focus_score, focus_state.face_present)
        } else {
            (stats.focus_score, false)
        }
    } else {
        (stats.focus_score, false)
    };

    PetStateResponse {
        mood: machine.mood,
        focus_score,
        total_focus_minutes: stats.total_focus_ms as f32 / 60000.0,
        is_vision_active: vision_running,
        face_detected,
    }
}

/// 宠物状态响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetStateResponse {
    /// 当前情绪
    pub mood: PetMood,
    /// 当前专注分数
    pub focus_score: f32,
    /// 今日累计专注时间（分钟）
    pub total_focus_minutes: f32,
    /// 视觉检测是否活跃
    pub is_vision_active: bool,
    /// 是否检测到人脸
    pub face_detected: bool,
}

/// 启动视觉检测
#[tauri::command]
pub async fn start_vision(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    {
        let mut running = state.vision_running.lock();
        if *running {
            return Err("Vision is already running".to_string());
        }
        *running = true;
    }

    tracing::info!("Starting vision detection...");

    // 获取资源目录路径
    let resource_path = app_handle
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {}", e))?;

    let model_path = resource_path
        .join("models")
        .join("blazeface.onnx")
        .to_string_lossy()
        .to_string();

    let anchors_path = resource_path
        .join("models")
        .join("anchors.npy")
        .to_string_lossy()
        .to_string();

    // 创建视觉处理器配置
    let config = VisionProcessorConfig {
        model_path,
        anchors_path: Some(anchors_path),
        detect_every_frame: false, // 隔帧检测以降低 CPU
        ..Default::default()
    };

    // 创建视觉处理器
    let processor = Arc::new(VisionProcessor::new(config));
    let focus_rx = processor.subscribe();

    // 启动处理器
    processor.start()?;

    // 保存处理器和接收器
    {
        *state.vision_processor.lock() = Some(processor.clone());
        *state.focus_state_rx.lock() = Some(focus_rx.clone());
    }

    // 启动状态更新任务
    let state_clone = state.inner().clone();
    let app_handle_clone = app_handle.clone();

    tokio::spawn(async move {
        let mut rx = focus_rx;

        while rx.changed().await.is_ok() {
            let focus_state = rx.borrow().clone();

            // 更新宠物状态机
            {
                let mut machine = state_clone.pet_state_machine.lock();
                let new_mood = machine.update(focus_state.focus_score, focus_state.face_present);

                // 如果状态改变，发送事件到前端
                if let Some(mood) = new_mood {
                    let _ = app_handle_clone.emit("pet_mood_changed", mood);
                }

                // 更新统计
                let mut stats = state_clone.focus_stats.lock();
                stats.focus_score = focus_state.focus_score;
                stats.current_mood = machine.mood;
                stats.focus_level = machine.focus_level;
                stats.total_focus_ms = machine.total_focus_ms;
            }

            // 发送专注状态事件
            let _ = app_handle_clone.emit("focus_state", &focus_state);
        }

        tracing::info!("Vision state update task ended");
    });

    tracing::info!("Vision detection started successfully");
    Ok(())
}

/// 停止视觉检测
#[tauri::command]
pub fn stop_vision(state: State<'_, AppState>) -> Result<(), String> {
    let mut running = state.vision_running.lock();
    if !*running {
        return Err("Vision is not running".to_string());
    }

    tracing::info!("Stopping vision detection...");

    // 停止处理器
    if let Some(ref processor) = *state.vision_processor.lock() {
        processor.stop();
    }

    // 清理状态
    *state.vision_processor.lock() = None;
    *state.focus_state_rx.lock() = None;
    *running = false;

    tracing::info!("Vision detection stopped");
    Ok(())
}

/// 触发手势事件（用于测试/Demo模式）
#[tauri::command]
pub fn trigger_gesture(gesture: String, state: State<'_, AppState>) -> Result<PetMood, String> {
    let gesture_type = match gesture.to_lowercase().as_str() {
        "wave" => GestureType::Wave,
        "heart" => GestureType::Heart,
        "ok" => GestureType::Ok,
        "thumbsup" | "thumbs_up" => GestureType::ThumbsUp,
        _ => return Err(format!("Unknown gesture: {}", gesture)),
    };

    tracing::info!("Gesture triggered: {:?}", gesture_type);

    // 更新宠物状态为互动模式
    let mut machine = state.pet_state_machine.lock();
    let new_mood = machine.on_gesture(gesture_type);

    Ok(new_mood)
}

/// 设置 Demo 模式的宠物状态（用于录屏展示）
#[tauri::command]
pub fn set_demo_mood(mood: String, state: State<'_, AppState>) -> Result<PetMood, String> {
    let new_mood = match mood.to_lowercase().as_str() {
        "idle" => PetMood::Idle,
        "happy" => PetMood::Happy,
        "excited" => PetMood::Excited,
        "sad" => PetMood::Sad,
        "sleepy" => PetMood::Sleepy,
        "interact" => PetMood::Interact,
        _ => return Err(format!("Unknown mood: {}", mood)),
    };

    let mut machine = state.pet_state_machine.lock();
    machine.mood = new_mood;

    tracing::info!("Demo mood set to: {:?}", new_mood);

    Ok(new_mood)
}

/// 获取今日专注统计
#[tauri::command]
pub fn get_focus_stats(state: State<'_, AppState>) -> FocusStats {
    state.focus_stats.lock().clone()
}

/// 重置今日统计
#[tauri::command]
pub fn reset_stats(state: State<'_, AppState>) {
    let mut stats = state.focus_stats.lock();
    stats.total_focus_ms = 0;

    let mut machine = state.pet_state_machine.lock();
    machine.reset_daily_stats();

    tracing::info!("Focus stats reset");
}

/// 获取视觉检测状态（详细信息）
#[tauri::command]
pub fn get_vision_status(state: State<'_, AppState>) -> VisionStatusResponse {
    let running = *state.vision_running.lock();

    let focus_state = if running {
        state.focus_state_rx.lock().as_ref().map(|rx| rx.borrow().clone())
    } else {
        None
    };

    VisionStatusResponse {
        is_running: running,
        focus_state,
    }
}

/// 视觉检测状态响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionStatusResponse {
    /// 是否正在运行
    pub is_running: bool,
    /// 当前专注状态
    pub focus_state: Option<FocusState>,
}
