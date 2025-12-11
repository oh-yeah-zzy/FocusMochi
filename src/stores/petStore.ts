/**
 * 宠物状态管理 Store
 * 使用 React 内置的 useReducer 模式，暂不引入 Zustand 依赖
 */

import { useCallback, useEffect, useReducer } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { PetMood, PetStateResponse, FocusStats, GestureType, FocusState } from '../types';

/** Store 状态 */
interface PetState {
  /** 当前情绪 */
  mood: PetMood;
  /** 专注分数 */
  focusScore: number;
  /** 今日专注时间（分钟） */
  totalFocusMinutes: number;
  /** 视觉检测是否活跃 */
  isVisionActive: boolean;
  /** 是否检测到人脸 */
  faceDetected: boolean;
  /** 是否正在加载 */
  isLoading: boolean;
  /** 错误信息 */
  error: string | null;
}

/** Action 类型 */
type PetAction =
  | { type: 'SET_STATE'; payload: PetStateResponse }
  | { type: 'SET_MOOD'; payload: PetMood }
  | { type: 'SET_LOADING'; payload: boolean }
  | { type: 'SET_ERROR'; payload: string | null }
  | { type: 'SET_VISION_ACTIVE'; payload: boolean };

/** 初始状态 */
const initialState: PetState = {
  mood: 'idle',
  focusScore: 0,
  totalFocusMinutes: 0,
  isVisionActive: false,
  faceDetected: false,
  isLoading: false,
  error: null,
};

/** Reducer */
function petReducer(state: PetState, action: PetAction): PetState {
  switch (action.type) {
    case 'SET_STATE':
      return {
        ...state,
        mood: action.payload.mood,
        focusScore: action.payload.focus_score,
        totalFocusMinutes: action.payload.total_focus_minutes,
        isVisionActive: action.payload.is_vision_active,
        faceDetected: action.payload.face_detected,
        isLoading: false,
        error: null,
      };
    case 'SET_MOOD':
      return { ...state, mood: action.payload };
    case 'SET_LOADING':
      return { ...state, isLoading: action.payload };
    case 'SET_ERROR':
      return { ...state, error: action.payload, isLoading: false };
    case 'SET_VISION_ACTIVE':
      return { ...state, isVisionActive: action.payload };
    default:
      return state;
  }
}

/**
 * 宠物状态管理 Hook
 */
export function usePetStore() {
  const [state, dispatch] = useReducer(petReducer, initialState);

  /** 获取当前状态 */
  const fetchState = useCallback(async () => {
    try {
      dispatch({ type: 'SET_LOADING', payload: true });
      const response = await invoke<PetStateResponse>('get_pet_state');
      dispatch({ type: 'SET_STATE', payload: response });
    } catch (err) {
      dispatch({ type: 'SET_ERROR', payload: String(err) });
    }
  }, []);

  /** 启动视觉检测 */
  const startVision = useCallback(async () => {
    try {
      await invoke('start_vision');
      dispatch({ type: 'SET_VISION_ACTIVE', payload: true });
    } catch (err) {
      dispatch({ type: 'SET_ERROR', payload: String(err) });
    }
  }, []);

  /** 停止视觉检测 */
  const stopVision = useCallback(async () => {
    try {
      await invoke('stop_vision');
      dispatch({ type: 'SET_VISION_ACTIVE', payload: false });
    } catch (err) {
      dispatch({ type: 'SET_ERROR', payload: String(err) });
    }
  }, []);

  /** 触发手势 */
  const triggerGesture = useCallback(async (gesture: GestureType) => {
    try {
      const newMood = await invoke<PetMood>('trigger_gesture', { gesture });
      dispatch({ type: 'SET_MOOD', payload: newMood });
    } catch (err) {
      dispatch({ type: 'SET_ERROR', payload: String(err) });
    }
  }, []);

  /** 设置 Demo 模式的情绪 */
  const setDemoMood = useCallback(async (mood: PetMood) => {
    try {
      await invoke<PetMood>('set_demo_mood', { mood });
      dispatch({ type: 'SET_MOOD', payload: mood });
    } catch (err) {
      dispatch({ type: 'SET_ERROR', payload: String(err) });
    }
  }, []);

  /** 获取专注统计 */
  const getFocusStats = useCallback(async (): Promise<FocusStats | null> => {
    try {
      return await invoke<FocusStats>('get_focus_stats');
    } catch (err) {
      dispatch({ type: 'SET_ERROR', payload: String(err) });
      return null;
    }
  }, []);

  /** 重置统计 */
  const resetStats = useCallback(async () => {
    try {
      await invoke('reset_stats');
      await fetchState();
    } catch (err) {
      dispatch({ type: 'SET_ERROR', payload: String(err) });
    }
  }, [fetchState]);

  // 定期刷新状态
  useEffect(() => {
    fetchState();
    const interval = setInterval(fetchState, 1000);
    return () => clearInterval(interval);
  }, [fetchState]);

  return {
    ...state,
    fetchState,
    startVision,
    stopVision,
    triggerGesture,
    setDemoMood,
    getFocusStats,
    resetStats,
  };
}
