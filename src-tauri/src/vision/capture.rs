//! 摄像头采集模块
//! 负责从摄像头捕获视频帧，支持真实摄像头和模拟模式

use image::RgbImage;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::watch;

/// 摄像头配置
#[derive(Debug, Clone)]
pub struct CameraConfig {
    /// 摄像头设备索引
    pub device_index: u32,
    /// 目标帧率
    pub target_fps: u32,
    /// 采集宽度
    pub width: u32,
    /// 采集高度
    pub height: u32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            device_index: 0,
            target_fps: 10, // 降低帧率以减少 CPU 占用
            width: 320,     // 使用较低分辨率
            height: 240,
        }
    }
}

/// 捕获的视频帧
#[derive(Debug, Clone)]
pub struct CapturedFrame {
    /// 帧宽度
    pub width: u32,
    /// 帧高度
    pub height: u32,
    /// RGB 数据 (height * width * 3)
    pub data: Vec<u8>,
    /// 时间戳（毫秒）
    pub timestamp_ms: u64,
}

impl CapturedFrame {
    /// 创建空帧
    pub fn empty() -> Self {
        Self {
            width: 0,
            height: 0,
            data: Vec::new(),
            timestamp_ms: 0,
        }
    }

    /// 检查是否为空帧
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// 转换为 image crate 的 RgbImage
    pub fn to_rgb_image(&self) -> Option<RgbImage> {
        if self.is_empty() {
            return None;
        }
        RgbImage::from_raw(self.width, self.height, self.data.clone())
    }
}

/// 摄像头采集器状态
#[derive(Debug, Clone)]
pub enum CaptureState {
    /// 未初始化
    Uninitialized,
    /// 正在运行
    Running,
    /// 已停止
    Stopped,
    /// 错误
    Error(String),
}

/// 摄像头采集器
///
/// 使用 tokio::sync::watch 通道发布最新帧，避免帧堆积
pub struct CameraCapture {
    config: CameraConfig,
    running: Arc<AtomicBool>,
    /// 帧发送端（内部使用）
    frame_tx: watch::Sender<CapturedFrame>,
    /// 帧接收端（供外部订阅）
    frame_rx: watch::Receiver<CapturedFrame>,
}

impl CameraCapture {
    /// 创建新的摄像头采集器
    pub fn new(config: CameraConfig) -> Self {
        let (frame_tx, frame_rx) = watch::channel(CapturedFrame::empty());
        Self {
            config,
            running: Arc::new(AtomicBool::new(false)),
            frame_tx,
            frame_rx,
        }
    }

    /// 获取帧接收器的克隆（用于订阅最新帧）
    pub fn subscribe(&self) -> watch::Receiver<CapturedFrame> {
        self.frame_rx.clone()
    }

    /// 启动摄像头采集
    ///
    /// 在后台线程中运行采集循环，通过 watch 通道发布帧
    pub fn start(&self) -> Result<(), String> {
        if self.running.load(Ordering::SeqCst) {
            return Err("Camera is already running".to_string());
        }

        let running = self.running.clone();
        let config = self.config.clone();
        let frame_tx = self.frame_tx.clone();

        running.store(true, Ordering::SeqCst);

        // 启动采集线程
        tokio::spawn(async move {
            tracing::info!("Camera capture starting with config: {:?}", config);

            // 计算帧间隔
            let frame_interval =
                std::time::Duration::from_millis(1000 / config.target_fps.max(1) as u64);

            #[cfg(feature = "vision")]
            {
                // 真实摄像头采集（需要 nokhwa）
                match Self::run_real_capture(&config, &running, &frame_tx, frame_interval).await {
                    Ok(_) => tracing::info!("Camera capture stopped normally"),
                    Err(e) => tracing::error!("Camera capture error: {}", e),
                }
            }

            #[cfg(not(feature = "vision"))]
            {
                // 模拟模式（用于开发测试）
                Self::run_mock_capture(&config, &running, &frame_tx, frame_interval).await;
            }

            running.store(false, Ordering::SeqCst);
            tracing::info!("Camera capture thread exited");
        });

        Ok(())
    }

    /// 停止采集
    pub fn stop(&self) {
        tracing::info!("Stopping camera capture...");
        self.running.store(false, Ordering::SeqCst);
    }

    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// 模拟采集循环（开发测试用）
    #[cfg(not(feature = "vision"))]
    async fn run_mock_capture(
        config: &CameraConfig,
        running: &Arc<AtomicBool>,
        frame_tx: &watch::Sender<CapturedFrame>,
        frame_interval: std::time::Duration,
    ) {
        tracing::info!("Running in MOCK mode (no real camera)");

        let mut frame_count = 0u64;

        while running.load(Ordering::SeqCst) {
            // 生成模拟帧（灰色图像，带一些变化模拟运动）
            let brightness = 128u8.wrapping_add((frame_count % 50) as u8);
            let data = vec![brightness; (config.width * config.height * 3) as usize];

            let frame = CapturedFrame {
                width: config.width,
                height: config.height,
                data,
                timestamp_ms: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
            };

            // 发送帧（watch 会自动丢弃旧帧）
            if frame_tx.send(frame).is_err() {
                tracing::warn!("All frame receivers dropped, stopping capture");
                break;
            }

            frame_count += 1;
            if frame_count % 100 == 0 {
                tracing::debug!("Mock capture: {} frames captured", frame_count);
            }

            tokio::time::sleep(frame_interval).await;
        }
    }

    /// 真实摄像头采集循环
    #[cfg(feature = "vision")]
    async fn run_real_capture(
        config: &CameraConfig,
        running: &Arc<AtomicBool>,
        frame_tx: &watch::Sender<CapturedFrame>,
        frame_interval: std::time::Duration,
    ) -> Result<(), String> {
        use nokhwa::pixel_format::RgbFormat;
        use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
        use nokhwa::Camera;

        // 创建摄像头请求格式
        let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);

        // 打开摄像头
        let index = CameraIndex::Index(config.device_index);
        let mut camera = Camera::new(index, requested)
            .map_err(|e| format!("Failed to open camera: {}", e))?;

        tracing::info!("Camera opened successfully");

        // 获取实际分辨率
        let resolution = camera.resolution();
        tracing::info!(
            "Camera resolution: {}x{}",
            resolution.width(),
            resolution.height()
        );

        // 开始采集
        camera
            .open_stream()
            .map_err(|e| format!("Failed to start camera stream: {}", e))?;

        let mut frame_count = 0u64;

        while running.load(Ordering::SeqCst) {
            // 获取帧
            match camera.frame() {
                Ok(buffer) => {
                    // 解码为 RGB
                    let decoded = buffer
                        .decode_image::<RgbFormat>()
                        .map_err(|e| format!("Failed to decode frame: {}", e))?;

                    // 调整大小到目标分辨率（如果需要）
                    let resized = if decoded.width() != config.width
                        || decoded.height() != config.height
                    {
                        image::imageops::resize(
                            &decoded,
                            config.width,
                            config.height,
                            image::imageops::FilterType::Triangle,
                        )
                    } else {
                        decoded
                    };

                    let frame = CapturedFrame {
                        width: config.width,
                        height: config.height,
                        data: resized.into_raw(),
                        timestamp_ms: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as u64,
                    };

                    if frame_tx.send(frame).is_err() {
                        tracing::warn!("All frame receivers dropped, stopping capture");
                        break;
                    }

                    frame_count += 1;
                    if frame_count % 100 == 0 {
                        tracing::debug!("Real capture: {} frames captured", frame_count);
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to capture frame: {}", e);
                }
            }

            tokio::time::sleep(frame_interval).await;
        }

        // 关闭摄像头
        camera.stop_stream().ok();
        tracing::info!("Camera stream stopped");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_config_default() {
        let config = CameraConfig::default();
        assert_eq!(config.device_index, 0);
        assert_eq!(config.target_fps, 10);
        assert_eq!(config.width, 320);
        assert_eq!(config.height, 240);
    }

    #[test]
    fn test_captured_frame_empty() {
        let frame = CapturedFrame::empty();
        assert!(frame.is_empty());
        assert!(frame.to_rgb_image().is_none());
    }
}
