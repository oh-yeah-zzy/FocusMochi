//! 宠物状态机
//! 定义宠物的各种情绪状态和状态转换规则

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// 宠物的情绪状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PetMood {
    /// 默认待机状态
    Idle,
    /// 用户专注中，宠物开心
    Happy,
    /// 长时间专注，宠物非常兴奋
    Excited,
    /// 用户分心，宠物伤心
    Sad,
    /// 用户离开，宠物睡觉
    Sleepy,
    /// 响应手势互动
    Interact,
}

impl Default for PetMood {
    fn default() -> Self {
        Self::Idle
    }
}

/// 专注状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FocusLevel {
    /// 用户不在
    Away,
    /// 分心中
    Distracted,
    /// 专注中
    Focused,
}

impl Default for FocusLevel {
    fn default() -> Self {
        Self::Away
    }
}

/// 手势类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GestureType {
    /// 挥手
    Wave,
    /// 比心
    Heart,
    /// OK 手势
    Ok,
    /// 竖大拇指
    ThumbsUp,
}

/// 宠物状态机配置
#[derive(Debug, Clone)]
pub struct PetStateConfig {
    /// 进入专注状态的阈值
    pub focus_enter_threshold: f32,
    /// 退出专注状态的阈值
    pub focus_exit_threshold: f32,
    /// 专注状态确认时间（秒）
    pub focus_confirm_duration: f32,
    /// 触发兴奋状态的连续专注时间（分钟）
    pub excited_focus_minutes: f32,
    /// 判定离开的时间（秒）
    pub away_timeout: f32,
    /// 手势互动持续时间（秒）
    pub interact_duration: f32,
}

impl Default for PetStateConfig {
    fn default() -> Self {
        Self {
            focus_enter_threshold: 0.75,
            focus_exit_threshold: 0.35,
            focus_confirm_duration: 3.0,
            excited_focus_minutes: 25.0,
            away_timeout: 5.0,
            interact_duration: 3.0,
        }
    }
}

/// 宠物状态机
/// 根据专注分数和手势事件管理宠物的情绪状态
pub struct PetStateMachine {
    /// 当前情绪
    pub mood: PetMood,
    /// 当前专注等级
    pub focus_level: FocusLevel,
    /// 配置参数
    config: PetStateConfig,
    /// 状态进入时间
    mood_entered_at: Instant,
    /// 专注开始时间
    focus_started_at: Option<Instant>,
    /// 最后一次检测到人脸的时间
    last_face_detected_at: Option<Instant>,
    /// 当前专注分数（EMA 平滑后）
    smoothed_focus_score: f32,
    /// EMA 平滑系数
    ema_alpha: f32,
    /// 互动前的状态（用于互动结束后恢复）
    mood_before_interact: Option<PetMood>,
    /// 累计专注时间（毫秒）
    pub total_focus_ms: u64,
}

impl PetStateMachine {
    /// 创建新的状态机
    pub fn new(config: PetStateConfig) -> Self {
        Self {
            mood: PetMood::Idle,
            focus_level: FocusLevel::Away,
            config,
            mood_entered_at: Instant::now(),
            focus_started_at: None,
            last_face_detected_at: None,
            smoothed_focus_score: 0.0,
            ema_alpha: 0.15,
            mood_before_interact: None,
            total_focus_ms: 0,
        }
    }

    /// 更新专注分数并返回是否状态发生变化
    ///
    /// # Arguments
    /// * `raw_focus_score` - 原始专注分数 (0.0 - 1.0)
    /// * `face_detected` - 是否检测到人脸
    ///
    /// # Returns
    /// 如果状态发生变化，返回新的状态；否则返回 None
    pub fn update(&mut self, raw_focus_score: f32, face_detected: bool) -> Option<PetMood> {
        let now = Instant::now();
        let old_mood = self.mood;

        // 更新人脸检测时间
        if face_detected {
            self.last_face_detected_at = Some(now);
        }

        // 检查是否离开
        if let Some(last_face) = self.last_face_detected_at {
            if now.duration_since(last_face).as_secs_f32() > self.config.away_timeout {
                self.transition_to(PetMood::Sleepy);
                self.focus_level = FocusLevel::Away;
                self.focus_started_at = None;
                return if old_mood != self.mood { Some(self.mood) } else { None };
            }
        } else {
            // 从未检测到人脸
            self.transition_to(PetMood::Sleepy);
            self.focus_level = FocusLevel::Away;
            return if old_mood != self.mood { Some(self.mood) } else { None };
        }

        // 如果正在互动中，检查是否应该结束互动
        if self.mood == PetMood::Interact {
            if now.duration_since(self.mood_entered_at).as_secs_f32() > self.config.interact_duration {
                // 恢复互动前的状态
                if let Some(prev_mood) = self.mood_before_interact.take() {
                    self.mood = prev_mood;
                    self.mood_entered_at = now;
                }
            }
            return if old_mood != self.mood { Some(self.mood) } else { None };
        }

        // EMA 平滑专注分数
        self.smoothed_focus_score = self.ema_alpha * raw_focus_score
            + (1.0 - self.ema_alpha) * self.smoothed_focus_score;

        // 更新专注等级（带滞后）
        let new_focus_level = self.determine_focus_level();

        // 根据专注等级更新宠物状态
        match new_focus_level {
            FocusLevel::Focused => {
                // 首次进入专注
                if self.focus_level != FocusLevel::Focused {
                    self.focus_started_at = Some(now);
                    self.focus_level = FocusLevel::Focused;
                }

                // 检查是否应该进入兴奋状态
                if let Some(start) = self.focus_started_at {
                    let focus_duration = now.duration_since(start);
                    let excited_threshold = Duration::from_secs_f32(
                        self.config.excited_focus_minutes * 60.0
                    );

                    if focus_duration >= excited_threshold {
                        self.transition_to(PetMood::Excited);
                    } else {
                        self.transition_to(PetMood::Happy);
                    }

                    // 累计专注时间
                    self.total_focus_ms += 66; // 约 15fps，每帧约 66ms
                }
            }
            FocusLevel::Distracted => {
                self.focus_level = FocusLevel::Distracted;
                self.focus_started_at = None;
                self.transition_to(PetMood::Sad);
            }
            FocusLevel::Away => {
                self.focus_level = FocusLevel::Away;
                self.focus_started_at = None;
                self.transition_to(PetMood::Sleepy);
            }
        }

        if old_mood != self.mood {
            Some(self.mood)
        } else {
            None
        }
    }

    /// 处理手势事件
    pub fn on_gesture(&mut self, gesture: GestureType) -> PetMood {
        // 保存当前状态
        if self.mood != PetMood::Interact {
            self.mood_before_interact = Some(self.mood);
        }

        self.mood = PetMood::Interact;
        self.mood_entered_at = Instant::now();

        tracing::info!("Gesture detected: {:?}, entering Interact mode", gesture);

        self.mood
    }

    /// 判断专注等级
    fn determine_focus_level(&self) -> FocusLevel {
        let score = self.smoothed_focus_score;

        match self.focus_level {
            FocusLevel::Focused => {
                // 当前是专注状态，只有分数低于退出阈值才退出
                if score < self.config.focus_exit_threshold {
                    FocusLevel::Distracted
                } else {
                    FocusLevel::Focused
                }
            }
            FocusLevel::Distracted | FocusLevel::Away => {
                // 当前不是专注状态，只有分数高于进入阈值才进入
                if score > self.config.focus_enter_threshold {
                    FocusLevel::Focused
                } else {
                    FocusLevel::Distracted
                }
            }
        }
    }

    /// 转换到新状态
    fn transition_to(&mut self, new_mood: PetMood) {
        if self.mood != new_mood {
            tracing::debug!("Pet mood: {:?} -> {:?}", self.mood, new_mood);
            self.mood = new_mood;
            self.mood_entered_at = Instant::now();
        }
    }

    /// 获取今日专注统计
    pub fn get_focus_stats(&self) -> FocusStats {
        FocusStats {
            total_focus_ms: self.total_focus_ms,
            current_mood: self.mood,
            focus_level: self.focus_level,
            focus_score: self.smoothed_focus_score,
        }
    }

    /// 重置今日统计
    pub fn reset_daily_stats(&mut self) {
        self.total_focus_ms = 0;
    }
}

/// 专注统计数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusStats {
    /// 累计专注时间（毫秒）
    pub total_focus_ms: u64,
    /// 当前宠物情绪
    pub current_mood: PetMood,
    /// 当前专注等级
    pub focus_level: FocusLevel,
    /// 当前专注分数
    pub focus_score: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let machine = PetStateMachine::new(PetStateConfig::default());
        assert_eq!(machine.mood, PetMood::Idle);
        assert_eq!(machine.focus_level, FocusLevel::Away);
    }

    #[test]
    fn test_focus_transition() {
        let mut machine = PetStateMachine::new(PetStateConfig::default());

        // 模拟持续高专注分数
        for _ in 0..100 {
            machine.update(0.9, true);
        }

        assert_eq!(machine.focus_level, FocusLevel::Focused);
        assert!(matches!(machine.mood, PetMood::Happy | PetMood::Excited));
    }
}
