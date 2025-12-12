//! 视觉处理器模块
//!
//! 整合摄像头采集、人脸检测和专注度计算，
//! 提供统一的视觉处理循环

use super::{
    BlazeFaceDetector, CameraCapture, CameraConfig, FocusCalculator, FocusState,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::watch;

/// 视觉处理器配置
#[derive(Debug, Clone)]
pub struct VisionProcessorConfig {
    /// 摄像头配置
    pub camera: CameraConfig,
    /// 模型路径
    pub model_path: String,
    /// 锚框路径（可选）
    pub anchors_path: Option<String>,
    /// 是否每帧都进行检测（false 则隔帧检测以降低 CPU）
    pub detect_every_frame: bool,
}

impl Default for VisionProcessorConfig {
    fn default() -> Self {
        Self {
            camera: CameraConfig::default(),
            model_path: "resources/models/blazeface.onnx".to_string(),
            anchors_path: Some("resources/models/anchors.npy".to_string()),
            detect_every_frame: false, // 默认隔帧检测
        }
    }
}

/// 视觉处理器
///
/// 管理完整的视觉处理流程：
/// 1. 摄像头采集帧
/// 2. 人脸检测
/// 3. 专注度计算
/// 4. 通过 watch 通道发布专注状态
pub struct VisionProcessor {
    config: VisionProcessorConfig,
    running: Arc<AtomicBool>,
    /// 专注状态发送端
    state_tx: watch::Sender<FocusState>,
    /// 专注状态接收端（供外部订阅）
    state_rx: watch::Receiver<FocusState>,
    /// 原始帧发送端（用于预览）
    frame_tx: watch::Sender<super::CapturedFrame>,
    /// 原始帧接收端（供外部订阅预览）
    frame_rx: watch::Receiver<super::CapturedFrame>,
}

impl VisionProcessor {
    /// 创建视觉处理器
    pub fn new(config: VisionProcessorConfig) -> Self {
        let (state_tx, state_rx) = watch::channel(FocusState::default());
        let (frame_tx, frame_rx) = watch::channel(super::CapturedFrame::empty());

        Self {
            config,
            running: Arc::new(AtomicBool::new(false)),
            state_tx,
            state_rx,
            frame_tx,
            frame_rx,
        }
    }

    /// 获取专注状态订阅器
    pub fn subscribe(&self) -> watch::Receiver<FocusState> {
        self.state_rx.clone()
    }

    /// 获取帧数据订阅器（用于预览）
    pub fn subscribe_frames(&self) -> watch::Receiver<super::CapturedFrame> {
        self.frame_rx.clone()
    }

    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// 启动视觉处理
    pub fn start(&self) -> Result<(), String> {
        if self.running.load(Ordering::SeqCst) {
            return Err("Vision processor is already running".to_string());
        }

        let running = self.running.clone();
        let config = self.config.clone();
        let state_tx = self.state_tx.clone();
        let frame_tx = self.frame_tx.clone();

        running.store(true, Ordering::SeqCst);

        tokio::spawn(async move {
            tracing::info!("Vision processor starting...");

            if let Err(e) = Self::run_processing_loop(&config, &running, &state_tx, &frame_tx).await {
                tracing::error!("Vision processing error: {}", e);
            }

            running.store(false, Ordering::SeqCst);
            tracing::info!("Vision processor stopped");
        });

        Ok(())
    }

    /// 停止视觉处理
    pub fn stop(&self) {
        tracing::info!("Stopping vision processor...");
        self.running.store(false, Ordering::SeqCst);
    }

    /// 运行处理循环
    async fn run_processing_loop(
        config: &VisionProcessorConfig,
        running: &Arc<AtomicBool>,
        state_tx: &watch::Sender<FocusState>,
        frame_tx: &watch::Sender<super::CapturedFrame>,
    ) -> Result<(), String> {
        // 1. 创建摄像头采集器
        let camera = CameraCapture::new(config.camera.clone());
        let mut frame_rx = camera.subscribe();

        // 2. 创建人脸检测器
        let mut detector = BlazeFaceDetector::new(
            &config.model_path,
            config.anchors_path.as_deref(),
        )
        .map_err(|e| format!("Failed to create face detector: {}", e))?;

        // 3. 创建专注度计算器
        let calculator = FocusCalculator::with_defaults();

        // 4. 启动摄像头
        camera.start().map_err(|e| format!("Failed to start camera: {}", e))?;

        tracing::info!("Vision processing loop started");

        let mut frame_count = 0u64;
        let mut last_focus_state = FocusState::default();

        // 5. 处理循环
        while running.load(Ordering::SeqCst) {
            // 等待新帧
            if frame_rx.changed().await.is_err() {
                tracing::warn!("Frame channel closed");
                break;
            }

            let frame = frame_rx.borrow().clone();

            // 跳过空帧
            if frame.is_empty() {
                continue;
            }

            frame_count += 1;

            if frame_count == 1 {
                tracing::info!("First frame captured: {}x{}", frame.width, frame.height);
            }

            // 转发帧用于预览（每 3 帧转发一次，约 3-4 fps，包含第一帧）
            if frame_count % 3 == 1 {
                let _ = frame_tx.send(frame.clone());
            }

            // 是否进行检测（隔帧检测以降低 CPU）
            let should_detect = config.detect_every_frame || (frame_count % 2 == 0);

            if should_detect {
                // 运行人脸检测
                match detector.detect(&frame.data, frame.width, frame.height) {
                    Ok(detections) => {
                        // 获取最大置信度的人脸
                        let primary_face = detections.first();

                        // 计算专注分数
                        let (focus_score, face_detected) = calculator.calculate(primary_face);

                        // 创建专注状态
                        let focus_state = FocusState::from_detection(primary_face, focus_score);

                        // 发布状态
                        if state_tx.send(focus_state.clone()).is_err() {
                            tracing::warn!("All state receivers dropped");
                            break;
                        }

                        last_focus_state = focus_state;

                        if frame_count % 50 == 0 {
                            tracing::debug!(
                                "Frame {}: face={}, score={:.2}",
                                frame_count,
                                face_detected,
                                focus_score
                            );
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Face detection error: {}", e);
                    }
                }
            } else {
                // 不检测时发送上一次的状态（更新时间戳）
                let mut state = last_focus_state.clone();
                state.timestamp_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;

                if state_tx.send(state).is_err() {
                    break;
                }
            }
        }

        // 停止摄像头
        camera.stop();

        Ok(())
    }
}

/// 创建默认配置的视觉处理器
pub fn create_default_processor() -> VisionProcessor {
    VisionProcessor::new(VisionProcessorConfig::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vision_processor_config_default() {
        let config = VisionProcessorConfig::default();
        assert!(!config.detect_every_frame);
        assert!(config.model_path.contains("blazeface"));
    }

    #[test]
    fn test_vision_processor_creation() {
        let processor = VisionProcessor::new(VisionProcessorConfig::default());
        assert!(!processor.is_running());
    }
}
