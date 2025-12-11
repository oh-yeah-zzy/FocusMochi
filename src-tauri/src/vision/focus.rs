//! 专注度计算模块
//!
//! 基于人脸检测结果（位置、大小、姿态）计算用户的专注程度
//! 主要考虑因素：
//! - 人脸是否存在
//! - 头部姿态（偏航、俯仰、翻滚角）
//! - 人脸在画面中的位置稳定性

use super::face::FaceDetection;
use serde::{Deserialize, Serialize};

/// 专注度计算器配置
#[derive(Debug, Clone)]
pub struct FocusCalculatorConfig {
    /// 人脸置信度权重
    pub face_confidence_weight: f32,
    /// 偏航角权重（左右转头）
    pub yaw_weight: f32,
    /// 俯仰角权重（上下点头）
    pub pitch_weight: f32,
    /// 翻滚角权重（歪头）
    pub roll_weight: f32,
    /// 偏航角最大值（度）- 超过此值视为完全分心
    pub max_yaw: f32,
    /// 俯仰角最大值（度）
    pub max_pitch: f32,
    /// 翻滚角最大值（度）
    pub max_roll: f32,
    /// 最小人脸置信度阈值
    pub min_face_confidence: f32,
    /// 人脸大小权重（用于判断是否靠近屏幕）
    pub face_size_weight: f32,
    /// 理想人脸大小比例（相对于画面）
    pub ideal_face_size: f32,
}

impl Default for FocusCalculatorConfig {
    fn default() -> Self {
        Self {
            face_confidence_weight: 0.3,
            yaw_weight: 0.25,
            pitch_weight: 0.2,
            roll_weight: 0.1,
            face_size_weight: 0.15,
            max_yaw: 30.0,
            max_pitch: 25.0,
            max_roll: 20.0,
            min_face_confidence: 0.5,
            ideal_face_size: 0.15, // 人脸占画面 15% 左右为理想
        }
    }
}

/// 专注度计算器
///
/// 根据人脸检测结果计算用户的专注程度
pub struct FocusCalculator {
    config: FocusCalculatorConfig,
}

impl FocusCalculator {
    /// 创建新的计算器
    pub fn new(config: FocusCalculatorConfig) -> Self {
        Self { config }
    }

    /// 使用默认配置创建
    pub fn with_defaults() -> Self {
        Self::new(FocusCalculatorConfig::default())
    }

    /// 计算专注分数
    ///
    /// # Arguments
    /// * `detection` - 人脸检测结果，None 表示未检测到人脸
    ///
    /// # Returns
    /// 返回 (专注分数, 是否检测到人脸)
    /// 专注分数范围 0.0 - 1.0，越高表示越专注
    pub fn calculate(&self, detection: Option<&FaceDetection>) -> (f32, bool) {
        let Some(face) = detection else {
            return (0.0, false);
        };

        // 检查人脸置信度是否足够
        if face.confidence < self.config.min_face_confidence {
            return (0.0, false);
        }

        // 1. 人脸置信度分量
        let conf_score = face.confidence;

        // 2. 偏航角分量（左右转头）
        let yaw = face.estimate_yaw();
        let yaw_normalized = (yaw.abs() / self.config.max_yaw).min(1.0);
        let yaw_score = 1.0 - yaw_normalized;

        // 3. 俯仰角分量（上下点头）
        let pitch = face.estimate_pitch();
        let pitch_normalized = (pitch.abs() / self.config.max_pitch).min(1.0);
        let pitch_score = 1.0 - pitch_normalized;

        // 4. 翻滚角分量（歪头）
        let roll = face.estimate_roll();
        let roll_normalized = (roll.abs() / self.config.max_roll).min(1.0);
        let roll_score = 1.0 - roll_normalized;

        // 5. 人脸大小分量（判断距离是否合适）
        let face_size = face.size();
        let size_diff = (face_size - self.config.ideal_face_size).abs();
        let size_score = (1.0 - size_diff / self.config.ideal_face_size).max(0.0);

        // 综合计算专注分数
        let focus_score = self.config.face_confidence_weight * conf_score
            + self.config.yaw_weight * yaw_score
            + self.config.pitch_weight * pitch_score
            + self.config.roll_weight * roll_score
            + self.config.face_size_weight * size_score;

        // 确保分数在 0-1 范围内
        let focus_score = focus_score.clamp(0.0, 1.0);

        tracing::trace!(
            "Focus calculation: conf={:.2}, yaw={:.1}({:.2}), pitch={:.1}({:.2}), roll={:.1}({:.2}), size={:.3}({:.2}) => {:.2}",
            conf_score, yaw, yaw_score, pitch, pitch_score, roll, roll_score, face_size, size_score, focus_score
        );

        (focus_score, true)
    }
}

/// 专注状态快照
///
/// 用于通过 watch 通道在线程间传递
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusState {
    /// 是否检测到人脸
    pub face_present: bool,
    /// 人脸置信度
    pub face_confidence: f32,
    /// 专注分数 (0.0 - 1.0)
    pub focus_score: f32,
    /// 头部偏航角（左右转头）
    pub yaw: f32,
    /// 头部俯仰角（上下点头）
    pub pitch: f32,
    /// 头部翻滚角（歪头）
    pub roll: f32,
    /// 时间戳（毫秒）
    pub timestamp_ms: u64,
}

impl Default for FocusState {
    fn default() -> Self {
        Self {
            face_present: false,
            face_confidence: 0.0,
            focus_score: 0.0,
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
            timestamp_ms: 0,
        }
    }
}

impl FocusState {
    /// 从人脸检测结果创建专注状态
    pub fn from_detection(detection: Option<&FaceDetection>, focus_score: f32) -> Self {
        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        match detection {
            Some(face) => Self {
                face_present: true,
                face_confidence: face.confidence,
                focus_score,
                yaw: face.estimate_yaw(),
                pitch: face.estimate_pitch(),
                roll: face.estimate_roll(),
                timestamp_ms,
            },
            None => Self {
                face_present: false,
                face_confidence: 0.0,
                focus_score: 0.0,
                yaw: 0.0,
                pitch: 0.0,
                roll: 0.0,
                timestamp_ms,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_focused_face() -> FaceDetection {
        FaceDetection {
            confidence: 0.95,
            bbox: (0.25, 0.15, 0.75, 0.85),
            landmarks: [
                (0.35, 0.35), // 右眼
                (0.65, 0.35), // 左眼
                (0.50, 0.55), // 鼻子
                (0.50, 0.75), // 嘴巴
                (0.20, 0.40), // 右耳
                (0.80, 0.40), // 左耳
            ],
        }
    }

    fn make_distracted_face() -> FaceDetection {
        FaceDetection {
            confidence: 0.8,
            bbox: (0.1, 0.1, 0.5, 0.5), // 偏左上角
            landmarks: [
                (0.15, 0.25), // 右眼 - 明显偏移
                (0.35, 0.20), // 左眼
                (0.20, 0.35), // 鼻子
                (0.25, 0.45), // 嘴巴
                (0.05, 0.30), // 右耳
                (0.40, 0.25), // 左耳
            ],
        }
    }

    #[test]
    fn test_focus_calculation_focused() {
        let calculator = FocusCalculator::with_defaults();
        let detection = make_focused_face();

        let (score, detected) = calculator.calculate(Some(&detection));
        assert!(detected);
        assert!(score > 0.6, "Expected high focus score, got {}", score);
    }

    #[test]
    fn test_focus_calculation_distracted() {
        let calculator = FocusCalculator::with_defaults();
        let detection = make_distracted_face();

        let (score, detected) = calculator.calculate(Some(&detection));
        assert!(detected);
        // 分心时分数应该较低
        assert!(score < 0.8, "Expected lower focus score, got {}", score);
    }

    #[test]
    fn test_focus_calculation_no_face() {
        let calculator = FocusCalculator::with_defaults();
        let (score, detected) = calculator.calculate(None);
        assert!(!detected);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_focus_calculation_low_confidence() {
        let calculator = FocusCalculator::with_defaults();
        let detection = FaceDetection {
            confidence: 0.3, // 低于阈值
            bbox: (0.25, 0.15, 0.75, 0.85),
            landmarks: [(0.5, 0.5); 6],
        };

        let (score, detected) = calculator.calculate(Some(&detection));
        assert!(!detected);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_focus_state_from_detection() {
        let detection = make_focused_face();
        let state = FocusState::from_detection(Some(&detection), 0.85);

        assert!(state.face_present);
        assert!((state.face_confidence - 0.95).abs() < 0.01);
        assert!((state.focus_score - 0.85).abs() < 0.01);
    }
}
