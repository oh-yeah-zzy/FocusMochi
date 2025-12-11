//! 视觉处理模块
//!
//! 负责摄像头采集、人脸检测、手势识别和专注度计算
//!
//! ## 模块结构
//!
//! - `capture`: 摄像头采集，支持真实摄像头和模拟模式
//! - `face`: BlazeFace 人脸检测，使用 ONNX Runtime
//! - `focus`: 专注度计算，基于人脸姿态估计
//!
//! ## 使用方式
//!
//! ```ignore
//! use vision::{CameraCapture, BlazeFaceDetector, FocusCalculator};
//!
//! // 创建摄像头采集器
//! let camera = CameraCapture::new(CameraConfig::default());
//! let frame_rx = camera.subscribe();
//! camera.start()?;
//!
//! // 创建人脸检测器
//! let detector = BlazeFaceDetector::new("models/blazeface.onnx", None)?;
//!
//! // 创建专注度计算器
//! let calculator = FocusCalculator::with_defaults();
//!
//! // 处理帧
//! while let Ok(frame) = frame_rx.changed().await {
//!     let detections = detector.detect(&frame.data, frame.width, frame.height)?;
//!     let (focus_score, face_detected) = calculator.calculate(detections.first());
//!     // ...
//! }
//! ```

pub mod capture;
pub mod face;
pub mod focus;
pub mod processor;

// 重新导出主要类型
pub use capture::{CameraCapture, CameraConfig, CapturedFrame};
pub use face::{BlazeFaceDetector, FaceDetection, FaceDetectorError, BLAZEFACE_INPUT_SIZE};
pub use focus::{FocusCalculator, FocusCalculatorConfig, FocusState};
pub use processor::{VisionProcessor, VisionProcessorConfig, create_default_processor};
