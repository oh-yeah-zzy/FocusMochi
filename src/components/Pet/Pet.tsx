/**
 * 宠物主组件
 * 渲染 Q版麻糬宠物，根据情绪状态切换动画
 */

import { useEffect, useState } from 'react';
import type { PetMood } from '../../types';
import { MochiSvg } from './MochiSvg';
import './Pet.css';

interface PetProps {
  /** 当前情绪 */
  mood: PetMood;
  /** 点击回调 */
  onClick?: () => void;
  /** 宠物大小 */
  size?: number;
}

/**
 * 宠物动画类名
 */
const MOOD_CLASSES: Record<PetMood, string> = {
  idle: 'pet-idle',
  happy: 'pet-happy',
  excited: 'pet-excited',
  sad: 'pet-sad',
  sleepy: 'pet-sleepy',
  interact: 'pet-interact',
};

/**
 * 情绪中文名称
 */
const MOOD_LABELS: Record<PetMood, string> = {
  idle: '待机',
  happy: '开心',
  excited: '超开心',
  sad: '伤心',
  sleepy: '睡觉',
  interact: '互动',
};

export function Pet({ mood, onClick, size = 140 }: PetProps) {
  const [isAnimating, setIsAnimating] = useState(false);

  // 情绪变化时触发动画
  useEffect(() => {
    setIsAnimating(true);
    const timer = setTimeout(() => setIsAnimating(false), 500);
    return () => clearTimeout(timer);
  }, [mood]);

  return (
    <div
      className={`pet-container ${MOOD_CLASSES[mood]} ${isAnimating ? 'pet-transitioning' : ''}`}
      onClick={onClick}
      role="button"
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          onClick?.();
        }
      }}
    >
      {/* 专注时的光环 */}
      {(mood === 'happy' || mood === 'excited') && (
        <div className="pet-aura" />
      )}

      {/* 宠物主体 - Q版麻糬 SVG */}
      <div className="pet-body">
        <MochiSvg mood={mood} size={size} />
      </div>

      {/* 表情指示器 */}
      <div className="pet-mood-indicator">
        <span className="mood-label">{MOOD_LABELS[mood]}</span>
      </div>
    </div>
  );
}

export default Pet;
