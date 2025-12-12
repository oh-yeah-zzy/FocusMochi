//! BlazeFace 人脸检测模块
//!
//! 使用 ONNX Runtime 加载 BlazeFace 模型进行人脸检测
//! 输入：128x128 RGB 图像
//! 输出：人脸边界框 + 6个关键点（眼睛、耳朵、鼻子、嘴巴）

use serde::{Deserialize, Serialize};

/// BlazeFace 模型期望的输入尺寸
pub const BLAZEFACE_INPUT_SIZE: u32 = 128;

/// 人脸检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceDetection {
    /// 人脸置信度 (0.0 - 1.0)
    pub confidence: f32,
    /// 人脸边界框 (x_min, y_min, x_max, y_max) - 归一化坐标 [0, 1]
    pub bbox: (f32, f32, f32, f32),
    /// 6个关键点坐标 [(x, y), ...] - 归一化坐标 [0, 1]
    /// 顺序：右眼、左眼、鼻子、嘴巴、右耳、左耳
    pub landmarks: [(f32, f32); 6],
}

impl FaceDetection {
    /// 计算人脸中心点
    pub fn center(&self) -> (f32, f32) {
        let (x1, y1, x2, y2) = self.bbox;
        ((x1 + x2) / 2.0, (y1 + y2) / 2.0)
    }

    /// 计算人脸大小（面积比例）
    pub fn size(&self) -> f32 {
        let (x1, y1, x2, y2) = self.bbox;
        (x2 - x1) * (y2 - y1)
    }

    /// 估算头部偏航角（左右转头）
    /// 基于眼睛中心与人脸中心的偏移
    pub fn estimate_yaw(&self) -> f32 {
        let (right_eye_x, _) = self.landmarks[0];
        let (left_eye_x, _) = self.landmarks[1];
        let (face_cx, _) = self.center();

        // 计算双眼中心
        let eyes_center_x = (right_eye_x + left_eye_x) / 2.0;

        // 偏移量转换为角度估计（简化计算）
        // 正值表示向右转，负值表示向左转
        let offset = eyes_center_x - face_cx;
        offset * 90.0 // 简化：假设最大偏移对应 45 度
    }

    /// 估算头部俯仰角（上下点头）
    /// 基于鼻子与眼睛的相对位置
    pub fn estimate_pitch(&self) -> f32 {
        let (_, right_eye_y) = self.landmarks[0];
        let (_, left_eye_y) = self.landmarks[1];
        let (_, nose_y) = self.landmarks[2];

        // 计算双眼中心 Y
        let eyes_center_y = (right_eye_y + left_eye_y) / 2.0;

        // 鼻子相对于眼睛的垂直距离
        let nose_offset = nose_y - eyes_center_y;

        // 转换为角度估计
        // 正值表示低头，负值表示抬头
        (nose_offset - 0.1) * 150.0 // 基准偏移约 0.1
    }

    /// 估算头部翻滚角（歪头）
    /// 基于双眼连线的倾斜程度
    pub fn estimate_roll(&self) -> f32 {
        let (right_eye_x, right_eye_y) = self.landmarks[0];
        let (left_eye_x, left_eye_y) = self.landmarks[1];

        // 计算双眼连线角度
        let dy = left_eye_y - right_eye_y;
        let dx = left_eye_x - right_eye_x;

        dy.atan2(dx).to_degrees()
    }
}

/// 人脸检测器错误
#[derive(Debug)]
pub enum FaceDetectorError {
    /// 模型加载失败
    ModelLoadError(String),
    /// 推理失败
    InferenceError(String),
    /// 图像处理失败
    ImageError(String),
}

impl std::fmt::Display for FaceDetectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FaceDetectorError::ModelLoadError(msg) => write!(f, "Model load error: {}", msg),
            FaceDetectorError::InferenceError(msg) => write!(f, "Inference error: {}", msg),
            FaceDetectorError::ImageError(msg) => write!(f, "Image error: {}", msg),
        }
    }
}

impl std::error::Error for FaceDetectorError {}

/// BlazeFace 人脸检测器
///
/// 模拟实现（无 vision feature）或真实 ONNX 推理（有 vision feature）
pub struct BlazeFaceDetector {
    /// 检测置信度阈值
    confidence_threshold: f32,
    /// NMS IoU 阈值
    #[allow(dead_code)]
    nms_threshold: f32,
    /// ONNX 会话（仅在 vision feature 启用时使用）
    #[cfg(feature = "vision")]
    session: ort::session::Session,
    /// 锚框数据
    #[cfg(feature = "vision")]
    anchors: ndarray::Array2<f32>,
}

impl BlazeFaceDetector {
    /// 创建检测器
    ///
    /// # Arguments
    /// * `model_path` - ONNX 模型文件路径
    /// * `anchors_path` - 锚框 npy 文件路径（可选，会尝试自动生成）
    #[cfg(feature = "vision")]
    pub fn new(model_path: &str, anchors_path: Option<&str>) -> Result<Self, FaceDetectorError> {
        use ort::session::{Session, builder::GraphOptimizationLevel};

        // 加载 ONNX 模型
        let session = Session::builder()
            .map_err(|e| FaceDetectorError::ModelLoadError(format!("Session builder error: {}", e)))?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| FaceDetectorError::ModelLoadError(format!("Optimization error: {}", e)))?
            .commit_from_file(model_path)
            .map_err(|e| FaceDetectorError::ModelLoadError(format!("Load model error: {}", e)))?;

        tracing::info!("BlazeFace model loaded from: {}", model_path);

        // 加载或生成锚框
        let anchors = if let Some(path) = anchors_path {
            Self::load_anchors(path)?
        } else {
            Self::generate_anchors()
        };

        Ok(Self {
            confidence_threshold: 0.5,
            nms_threshold: 0.3,
            session,
            anchors,
        })
    }

    /// 模拟模式创建（无真实模型）
    #[cfg(not(feature = "vision"))]
    pub fn new(_model_path: &str, _anchors_path: Option<&str>) -> Result<Self, FaceDetectorError> {
        tracing::info!("BlazeFace detector created in MOCK mode");
        Ok(Self {
            confidence_threshold: 0.5,
            nms_threshold: 0.3,
        })
    }

    /// 设置置信度阈值
    pub fn set_confidence_threshold(&mut self, threshold: f32) {
        self.confidence_threshold = threshold.clamp(0.0, 1.0);
    }

    /// 检测人脸
    ///
    /// # Arguments
    /// * `image_data` - RGB 图像数据
    /// * `width` - 图像宽度
    /// * `height` - 图像高度
    ///
    /// # Returns
    /// 检测到的人脸列表（按置信度降序排列）
    #[cfg(feature = "vision")]
    pub fn detect(
        &mut self,
        image_data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<Vec<FaceDetection>, FaceDetectorError> {
        use image::{ImageBuffer, Rgb};
        use ndarray::Array4;

        // 1. 将原始数据转换为 RgbImage
        let img: ImageBuffer<Rgb<u8>, Vec<u8>> =
            ImageBuffer::from_raw(width, height, image_data.to_vec())
                .ok_or_else(|| FaceDetectorError::ImageError("Invalid image data".to_string()))?;

        // 2. 调整大小到 128x128
        let resized = image::imageops::resize(
            &img,
            BLAZEFACE_INPUT_SIZE,
            BLAZEFACE_INPUT_SIZE,
            image::imageops::FilterType::Triangle,
        );

        // 3. 归一化到 [-1, 1] 并转换为 NCHW 格式
        let mut input_tensor = Array4::<f32>::zeros((1, 3, 128, 128));
        for y in 0..128 {
            for x in 0..128 {
                let pixel = resized.get_pixel(x, y);
                input_tensor[[0, 0, y as usize, x as usize]] = (pixel[0] as f32 / 127.5) - 1.0;
                input_tensor[[0, 1, y as usize, x as usize]] = (pixel[1] as f32 / 127.5) - 1.0;
                input_tensor[[0, 2, y as usize, x as usize]] = (pixel[2] as f32 / 127.5) - 1.0;
            }
        }

        // 4. 运行推理（ort 2.0: from_array 需要 owned array）
        let input_value = ort::value::Value::from_array(input_tensor)
            .map_err(|e| FaceDetectorError::InferenceError(format!("Input tensor error: {}", e)))?;

        let outputs = self
            .session
            .run(ort::inputs![input_value])
            .map_err(|e| FaceDetectorError::InferenceError(format!("Inference error: {}", e)))?;

        // 5. 解析输出
        // BlazeFace 输出: regressors [1, 896, 16] 和 classificators [1, 896, 1]
        // ort 2.0: try_extract_tensor 返回 (&Shape, &[T])
        let (_, regressors_data) = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|e| FaceDetectorError::InferenceError(format!("Extract regressors error: {}", e)))?;
        let (_, classificators_data) = outputs[1]
            .try_extract_tensor::<f32>()
            .map_err(|e| FaceDetectorError::InferenceError(format!("Extract classificators error: {}", e)))?;

        // 6. 解码检测结果
        // 输出形状: regressors [1, 896, 16] -> 扁平为 [896 * 16]
        //          classificators [1, 896, 1] -> 扁平为 [896]
        let mut detections = Vec::new();

        for i in 0..896 {
            // Sigmoid 转换置信度
            // classificators 是 [1, 896, 1] 扁平后索引为 i
            let score = 1.0 / (1.0 + (-classificators_data[i]).exp());

            if score > self.confidence_threshold {
                // 解码边界框（相对于锚框）
                let anchor_x = self.anchors[[i, 0]];
                let anchor_y = self.anchors[[i, 1]];

                // regressors 是 [1, 896, 16] 扁平后，第 i 个检测框从 i * 16 开始
                let reg_offset = i * 16;
                let cx = anchor_x + regressors_data[reg_offset] / 128.0;
                let cy = anchor_y + regressors_data[reg_offset + 1] / 128.0;
                let w = regressors_data[reg_offset + 2] / 128.0;
                let h = regressors_data[reg_offset + 3] / 128.0;

                let x1 = (cx - w / 2.0).clamp(0.0, 1.0);
                let y1 = (cy - h / 2.0).clamp(0.0, 1.0);
                let x2 = (cx + w / 2.0).clamp(0.0, 1.0);
                let y2 = (cy + h / 2.0).clamp(0.0, 1.0);

                // 解码 6 个关键点
                let mut landmarks = [(0.0f32, 0.0f32); 6];
                for j in 0..6 {
                    let lx = anchor_x + regressors_data[reg_offset + 4 + j * 2] / 128.0;
                    let ly = anchor_y + regressors_data[reg_offset + 4 + j * 2 + 1] / 128.0;
                    landmarks[j] = (lx.clamp(0.0, 1.0), ly.clamp(0.0, 1.0));
                }

                detections.push(FaceDetection {
                    confidence: score,
                    bbox: (x1, y1, x2, y2),
                    landmarks,
                });
            }
        }

        // 7. NMS（非极大值抑制）
        detections.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        let detections = self.nms(detections);

        Ok(detections)
    }

    /// 模拟检测（无 vision feature）
    #[cfg(not(feature = "vision"))]
    pub fn detect(
        &mut self,
        _image_data: &[u8],
        _width: u32,
        _height: u32,
    ) -> Result<Vec<FaceDetection>, FaceDetectorError> {
        // 返回模拟的人脸检测结果
        // 模拟一个正在专注的用户
        Ok(vec![FaceDetection {
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
        }])
    }

    /// 非极大值抑制
    #[allow(dead_code)]
    fn nms(&self, detections: Vec<FaceDetection>) -> Vec<FaceDetection> {
        if detections.is_empty() {
            return detections;
        }

        let mut keep = Vec::new();
        let mut suppressed = vec![false; detections.len()];

        for i in 0..detections.len() {
            if suppressed[i] {
                continue;
            }

            keep.push(detections[i].clone());

            for j in (i + 1)..detections.len() {
                if suppressed[j] {
                    continue;
                }

                let iou = Self::calculate_iou(&detections[i].bbox, &detections[j].bbox);
                if iou > self.nms_threshold {
                    suppressed[j] = true;
                }
            }
        }

        keep
    }

    /// 计算 IoU
    fn calculate_iou(
        box1: &(f32, f32, f32, f32),
        box2: &(f32, f32, f32, f32),
    ) -> f32 {
        let (x1_1, y1_1, x2_1, y2_1) = *box1;
        let (x1_2, y1_2, x2_2, y2_2) = *box2;

        let inter_x1 = x1_1.max(x1_2);
        let inter_y1 = y1_1.max(y1_2);
        let inter_x2 = x2_1.min(x2_2);
        let inter_y2 = y2_1.min(y2_2);

        let inter_w = (inter_x2 - inter_x1).max(0.0);
        let inter_h = (inter_y2 - inter_y1).max(0.0);
        let inter_area = inter_w * inter_h;

        let area1 = (x2_1 - x1_1) * (y2_1 - y1_1);
        let area2 = (x2_2 - x1_2) * (y2_2 - y1_2);

        inter_area / (area1 + area2 - inter_area + 1e-6)
    }

    /// 生成 BlazeFace 锚框
    #[cfg(feature = "vision")]
    fn generate_anchors() -> ndarray::Array2<f32> {
        use ndarray::Array2;

        // BlazeFace 使用的锚框配置
        // 两个特征图层级：16x16 和 8x8
        let strides = [8, 16];
        let anchor_counts = [2, 6]; // 每个位置的锚框数量

        let mut anchors = Vec::new();

        for (stride, &count) in strides.iter().zip(anchor_counts.iter()) {
            let grid_size = 128 / stride;
            for y in 0..grid_size {
                for x in 0..grid_size {
                    for _ in 0..count {
                        let anchor_x = (x as f32 + 0.5) / grid_size as f32;
                        let anchor_y = (y as f32 + 0.5) / grid_size as f32;
                        anchors.push([anchor_x, anchor_y]);
                    }
                }
            }
        }

        Array2::from_shape_vec((896, 2), anchors.into_iter().flatten().collect()).unwrap()
    }

    /// 从 npy 文件加载锚框
    #[cfg(feature = "vision")]
    fn load_anchors(path: &str) -> Result<ndarray::Array2<f32>, FaceDetectorError> {
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(path)
            .map_err(|e| FaceDetectorError::ModelLoadError(format!("Open anchors file error: {}", e)))?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|e| FaceDetectorError::ModelLoadError(format!("Read anchors file error: {}", e)))?;

        // 简单解析 npy 格式（假设是 float32，shape [896, 2]）
        // npy header 通常是 ~100 字节
        // 这里使用简化解析，实际应用中可以使用 ndarray-npy crate
        let header_len = buffer.iter().position(|&b| b == b'\n').unwrap_or(80) + 1;
        let data_start = if buffer[header_len..].starts_with(&[0x0A]) {
            header_len + 1
        } else {
            header_len
        };

        // 跳过第二个换行符（如果存在）
        let data_start = buffer[data_start..]
            .iter()
            .position(|&b| b != 0x0A && b != 0x20)
            .map(|p| data_start + p)
            .unwrap_or(data_start);

        let float_data: Vec<f32> = buffer[data_start..]
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        if float_data.len() != 896 * 2 {
            // 如果解析失败，使用生成的锚框
            tracing::warn!(
                "Anchors file parsing failed (got {} floats, expected 1792), using generated anchors",
                float_data.len()
            );
            return Ok(Self::generate_anchors());
        }

        ndarray::Array2::from_shape_vec((896, 2), float_data)
            .map_err(|e| FaceDetectorError::ModelLoadError(format!("Create anchors array error: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_face_detection_center() {
        let detection = FaceDetection {
            confidence: 0.9,
            bbox: (0.2, 0.1, 0.8, 0.9),
            landmarks: [(0.0, 0.0); 6],
        };
        let (cx, cy) = detection.center();
        assert!((cx - 0.5).abs() < 0.001);
        assert!((cy - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_face_detection_size() {
        let detection = FaceDetection {
            confidence: 0.9,
            bbox: (0.2, 0.2, 0.8, 0.8),
            landmarks: [(0.0, 0.0); 6],
        };
        let size = detection.size();
        assert!((size - 0.36).abs() < 0.001);
    }

    #[test]
    fn test_iou_calculation() {
        // 完全重叠
        let iou = BlazeFaceDetector::calculate_iou(
            &(0.0, 0.0, 1.0, 1.0),
            &(0.0, 0.0, 1.0, 1.0),
        );
        assert!((iou - 1.0).abs() < 0.001);

        // 无重叠
        let iou = BlazeFaceDetector::calculate_iou(
            &(0.0, 0.0, 0.5, 0.5),
            &(0.6, 0.6, 1.0, 1.0),
        );
        assert!(iou < 0.001);
    }
}
