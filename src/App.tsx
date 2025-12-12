/**
 * FocusMochi - AI Desktop Pet
 * 主应用组件
 */

import { useEffect, useCallback } from 'react';
import { Pet } from './components/Pet';
import { CameraPreview } from './components/CameraPreview';
import { usePetStore } from './stores/petStore';
import type { PetMood, GestureType } from './types';
import './App.css';

function App() {
  const {
    mood,
    focusScore,
    totalFocusMinutes,
    isVisionActive,
    faceDetected,
    error,
    triggerGesture,
    setDemoMood,
    startVision,
    stopVision,
  } = usePetStore();

  // 键盘快捷键（Demo 模式）
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // 数字键 1-6 切换情绪状态
      const moodMap: Record<string, PetMood> = {
        '1': 'idle',
        '2': 'happy',
        '3': 'excited',
        '4': 'sad',
        '5': 'sleepy',
        '6': 'interact',
      };

      // 字母键触发手势
      const gestureMap: Record<string, GestureType> = {
        'w': 'wave',
        'h': 'heart',
        'o': 'ok',
        't': 'thumbsup',
      };

      const key = e.key.toLowerCase();

      if (moodMap[key]) {
        setDemoMood(moodMap[key]);
      } else if (gestureMap[key]) {
        triggerGesture(gestureMap[key]);
      } else if (key === 'v') {
        // V 键切换视觉检测
        if (isVisionActive) {
          stopVision();
        } else {
          startVision();
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [setDemoMood, triggerGesture, isVisionActive, startVision, stopVision]);

  // 点击宠物触发互动
  const handlePetClick = useCallback(() => {
    triggerGesture('wave');
  }, [triggerGesture]);

  // 切换视觉检测
  const handleVisionToggle = useCallback(() => {
    if (isVisionActive) {
      stopVision();
    } else {
      startVision();
    }
  }, [isVisionActive, startVision, stopVision]);

  return (
    <main className="app-container">
      {/* 摄像头预览（视觉检测开启时显示） */}
      <CameraPreview visible={isVisionActive} />

      {/* 宠物主体 */}
      <Pet mood={mood} onClick={handlePetClick} />

      {/* 调试信息（开发模式显示） */}
      {import.meta.env.DEV && (
        <div className="debug-panel">
          <div className="debug-item">
            <span className="debug-label">Mood:</span>
            <span className="debug-value">{mood}</span>
          </div>
          <div className="debug-item">
            <span className="debug-label">Focus:</span>
            <span className="debug-value">{(focusScore * 100).toFixed(0)}%</span>
          </div>
          <div className="debug-item">
            <span className="debug-label">Today:</span>
            <span className="debug-value">{totalFocusMinutes.toFixed(1)}min</span>
          </div>
          <div className="debug-item">
            <span className="debug-label">Vision:</span>
            <span className={`debug-value ${isVisionActive ? 'active' : ''}`}>
              {isVisionActive ? 'ON' : 'OFF'}
            </span>
          </div>
          {isVisionActive && (
            <div className="debug-item">
              <span className="debug-label">Face:</span>
              <span className={`debug-value ${faceDetected ? 'active' : 'warning'}`}>
                {faceDetected ? '✓' : '✗'}
              </span>
            </div>
          )}
          {error && (
            <div className="debug-error">{error}</div>
          )}
          <div className="debug-help">
            <small>
              Keys: 1-6 mood | W/H/O/T gesture | V vision
            </small>
          </div>
          {/* 摄像头控制按钮 */}
          <button
            className={`vision-toggle-btn ${isVisionActive ? 'active' : ''}`}
            onClick={handleVisionToggle}
            title={isVisionActive ? '停止检测' : '启动检测'}
          >
            {isVisionActive ? '📷 停止' : '📷 启动'}
          </button>
        </div>
      )}
    </main>
  );
}

export default App;
