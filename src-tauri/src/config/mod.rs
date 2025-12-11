//! 配置管理模块
//! 加载和保存应用配置

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 摄像头设置
    pub camera: CameraSettings,
    /// 专注检测设置
    pub focus: FocusSettings,
    /// 宠物设置
    pub pet: PetSettings,
    /// 界面设置
    pub ui: UiSettings,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            camera: CameraSettings::default(),
            focus: FocusSettings::default(),
            pet: PetSettings::default(),
            ui: UiSettings::default(),
        }
    }
}

/// 摄像头设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraSettings {
    /// 摄像头设备索引
    pub device_index: u32,
    /// 目标帧率
    pub fps: u32,
    /// 是否启用摄像头
    pub enabled: bool,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            device_index: 0,
            fps: 15,
            enabled: true,
        }
    }
}

/// 专注检测设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusSettings {
    /// 进入专注状态的阈值 (0.0 - 1.0)
    pub enter_threshold: f32,
    /// 退出专注状态的阈值 (0.0 - 1.0)
    pub exit_threshold: f32,
    /// 状态确认时间（秒）
    pub confirm_duration: f32,
    /// 判定离开的超时时间（秒）
    pub away_timeout: f32,
    /// EMA 平滑系数
    pub ema_alpha: f32,
}

impl Default for FocusSettings {
    fn default() -> Self {
        Self {
            enter_threshold: 0.75,
            exit_threshold: 0.35,
            confirm_duration: 3.0,
            away_timeout: 5.0,
            ema_alpha: 0.15,
        }
    }
}

/// 宠物设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetSettings {
    /// 触发兴奋状态的连续专注时间（分钟）
    pub excited_focus_minutes: f32,
    /// 手势互动持续时间（秒）
    pub interact_duration: f32,
    /// 启用手势识别
    pub gesture_enabled: bool,
}

impl Default for PetSettings {
    fn default() -> Self {
        Self {
            excited_focus_minutes: 25.0,
            interact_duration: 3.0,
            gesture_enabled: true,
        }
    }
}

/// 界面设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSettings {
    /// 宠物窗口位置 X
    pub pet_x: i32,
    /// 宠物窗口位置 Y
    pub pet_y: i32,
    /// 宠物大小缩放
    pub pet_scale: f32,
    /// 是否置顶显示
    pub always_on_top: bool,
    /// 开机自启动
    pub auto_start: bool,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            pet_x: 100,
            pet_y: 100,
            pet_scale: 1.0,
            always_on_top: true,
            auto_start: false,
        }
    }
}

impl AppConfig {
    /// 从文件加载配置
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        let config: Self = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// 保存配置到文件
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// 加载或创建默认配置
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        Self::load(&path).unwrap_or_else(|_| {
            let config = Self::default();
            // 尝试保存默认配置
            let _ = config.save(&path);
            config
        })
    }
}

/// 配置错误
#[derive(Debug)]
pub enum ConfigError {
    IoError(std::io::Error),
    ParseError(serde_json::Error),
}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::IoError(err)
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(err: serde_json::Error) -> Self {
        ConfigError::ParseError(err)
    }
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IoError(e) => write!(f, "IO error: {}", e),
            ConfigError::ParseError(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.camera.fps, 15);
        assert_eq!(config.focus.enter_threshold, 0.75);
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.camera.fps, config.camera.fps);
    }
}
