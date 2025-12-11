/**
 * FocusMochi 类型定义
 */

/** 宠物情绪状态 */
export type PetMood =
  | 'idle'      // 默认待机
  | 'happy'     // 专注中，开心
  | 'excited'   // 长时间专注，非常兴奋
  | 'sad'       // 分心，伤心
  | 'sleepy'    // 离开，睡觉
  | 'interact'; // 响应手势互动

/** 专注等级 */
export type FocusLevel =
  | 'away'       // 用户不在
  | 'distracted' // 分心中
  | 'focused';   // 专注中

/** 手势类型 */
export type GestureType =
  | 'wave'      // 挥手
  | 'heart'     // 比心
  | 'ok'        // OK 手势
  | 'thumbsup'; // 竖大拇指

/** 宠物状态响应 */
export interface PetStateResponse {
  /** 当前情绪 */
  mood: PetMood;
  /** 当前专注分数 (0-1) */
  focus_score: number;
  /** 今日累计专注时间（分钟） */
  total_focus_minutes: number;
  /** 视觉检测是否活跃 */
  is_vision_active: boolean;
  /** 是否检测到人脸 */
  face_detected: boolean;
}

/** 专注状态（来自视觉检测） */
export interface FocusState {
  /** 是否检测到人脸 */
  face_present: boolean;
  /** 人脸置信度 */
  face_confidence: number;
  /** 专注分数 (0-1) */
  focus_score: number;
  /** 头部偏航角（左右转头） */
  yaw: number;
  /** 头部俯仰角（上下点头） */
  pitch: number;
  /** 头部翻滚角（歪头） */
  roll: number;
  /** 时间戳（毫秒） */
  timestamp_ms: number;
}

/** 视觉检测状态响应 */
export interface VisionStatusResponse {
  /** 是否正在运行 */
  is_running: boolean;
  /** 当前专注状态 */
  focus_state: FocusState | null;
}

/** 专注统计 */
export interface FocusStats {
  /** 累计专注时间（毫秒） */
  total_focus_ms: number;
  /** 当前宠物情绪 */
  current_mood: PetMood;
  /** 当前专注等级 */
  focus_level: FocusLevel;
  /** 当前专注分数 */
  focus_score: number;
}

/** 宠物动画帧配置 */
export interface PetAnimationConfig {
  /** 动画帧图片 */
  frames: string[];
  /** 帧率 (fps) */
  frameRate: number;
  /** 是否循环 */
  loop: boolean;
}

/** 宠物动画集合 */
export interface PetAnimations {
  idle: PetAnimationConfig;
  happy: PetAnimationConfig;
  excited: PetAnimationConfig;
  sad: PetAnimationConfig;
  sleepy: PetAnimationConfig;
  interact: PetAnimationConfig;
}
