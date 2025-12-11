/**
 * Q版麻糬 SVG 组件
 * 根据不同情绪状态显示不同的表情
 */

import type { PetMood } from '../../types';

interface MochiSvgProps {
  mood: PetMood;
  size?: number;
}

/**
 * 麻糬主体 SVG
 */
export function MochiSvg({ mood, size = 120 }: MochiSvgProps) {
  // 根据情绪获取表情组件
  const Face = MOOD_FACES[mood];

  // 根据情绪获取身体颜色
  const bodyColor = MOOD_COLORS[mood];
  const bodyColorLight = MOOD_COLORS_LIGHT[mood];

  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 120 120"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      className={`mochi-svg mochi-${mood}`}
    >
      {/* 阴影 */}
      <ellipse
        cx="60"
        cy="108"
        rx="35"
        ry="8"
        fill="rgba(0,0,0,0.1)"
        className="mochi-shadow"
      />

      {/* 身体 - 麻糬形状 */}
      <ellipse
        cx="60"
        cy="65"
        rx="45"
        ry="40"
        fill={bodyColor}
        className="mochi-body"
      />

      {/* 身体高光 */}
      <ellipse
        cx="45"
        cy="50"
        rx="20"
        ry="15"
        fill={bodyColorLight}
        opacity="0.6"
        className="mochi-highlight"
      />

      {/* 腮红 */}
      {(mood === 'happy' || mood === 'excited' || mood === 'interact') && (
        <>
          <ellipse cx="30" cy="70" rx="8" ry="5" fill="#FFB6C1" opacity="0.7" />
          <ellipse cx="90" cy="70" rx="8" ry="5" fill="#FFB6C1" opacity="0.7" />
        </>
      )}

      {/* 表情 */}
      <Face />

      {/* 互动时的爱心 */}
      {mood === 'interact' && <InteractHearts />}

      {/* 兴奋时的星星 */}
      {mood === 'excited' && <ExcitedStars />}

      {/* 睡觉时的 Zzz */}
      {mood === 'sleepy' && <SleepyZzz />}

      {/* 伤心时的汗滴 */}
      {mood === 'sad' && <SadSweat />}
    </svg>
  );
}

// ========== 表情组件 ==========

/** Idle 表情 - 平静 */
function IdleFace() {
  return (
    <g className="mochi-face">
      {/* 眼睛 */}
      <ellipse cx="45" cy="60" rx="5" ry="6" fill="#333" />
      <ellipse cx="75" cy="60" rx="5" ry="6" fill="#333" />
      {/* 眼睛高光 */}
      <circle cx="47" cy="58" r="2" fill="white" />
      <circle cx="77" cy="58" r="2" fill="white" />
      {/* 嘴巴 - 小圆点 */}
      <ellipse cx="60" cy="78" rx="3" ry="2" fill="#666" />
    </g>
  );
}

/** Happy 表情 - 开心 */
function HappyFace() {
  return (
    <g className="mochi-face">
      {/* 眼睛 - 弯弯的 */}
      <path
        d="M38 60 Q45 52 52 60"
        stroke="#333"
        strokeWidth="3"
        strokeLinecap="round"
        fill="none"
      />
      <path
        d="M68 60 Q75 52 82 60"
        stroke="#333"
        strokeWidth="3"
        strokeLinecap="round"
        fill="none"
      />
      {/* 嘴巴 - 微笑 */}
      <path
        d="M50 75 Q60 85 70 75"
        stroke="#333"
        strokeWidth="2.5"
        strokeLinecap="round"
        fill="none"
      />
    </g>
  );
}

/** Excited 表情 - 非常开心 */
function ExcitedFace() {
  return (
    <g className="mochi-face">
      {/* 眼睛 - 星星眼 */}
      <polygon
        points="45,55 47,60 52,60 48,64 50,70 45,66 40,70 42,64 38,60 43,60"
        fill="#FFD700"
        stroke="#333"
        strokeWidth="1"
      />
      <polygon
        points="75,55 77,60 82,60 78,64 80,70 75,66 70,70 72,64 68,60 73,60"
        fill="#FFD700"
        stroke="#333"
        strokeWidth="1"
      />
      {/* 嘴巴 - 大笑 */}
      <path
        d="M45 72 Q60 90 75 72"
        stroke="#333"
        strokeWidth="2"
        fill="#FF9999"
      />
    </g>
  );
}

/** Sad 表情 - 伤心 */
function SadFace() {
  return (
    <g className="mochi-face">
      {/* 眼睛 - 下垂 */}
      <ellipse cx="45" cy="62" rx="5" ry="6" fill="#333" />
      <ellipse cx="75" cy="62" rx="5" ry="6" fill="#333" />
      {/* 眉毛 - 八字眉 */}
      <path
        d="M35 50 L50 55"
        stroke="#333"
        strokeWidth="2"
        strokeLinecap="round"
      />
      <path
        d="M85 50 L70 55"
        stroke="#333"
        strokeWidth="2"
        strokeLinecap="round"
      />
      {/* 嘴巴 - 向下 */}
      <path
        d="M50 82 Q60 74 70 82"
        stroke="#333"
        strokeWidth="2.5"
        strokeLinecap="round"
        fill="none"
      />
    </g>
  );
}

/** Sleepy 表情 - 睡觉 */
function SleepyFace() {
  return (
    <g className="mochi-face">
      {/* 眼睛 - 闭眼线 */}
      <path
        d="M38 62 L52 62"
        stroke="#333"
        strokeWidth="2.5"
        strokeLinecap="round"
      />
      <path
        d="M68 62 L82 62"
        stroke="#333"
        strokeWidth="2.5"
        strokeLinecap="round"
      />
      {/* 嘴巴 - 小 O */}
      <ellipse cx="60" cy="78" rx="4" ry="5" fill="#333" />
    </g>
  );
}

/** Interact 表情 - 互动 */
function InteractFace() {
  return (
    <g className="mochi-face">
      {/* 眼睛 - 闪亮 */}
      <ellipse cx="45" cy="58" rx="6" ry="7" fill="#333" />
      <ellipse cx="75" cy="58" rx="6" ry="7" fill="#333" />
      {/* 高光 */}
      <circle cx="48" cy="56" r="3" fill="white" />
      <circle cx="78" cy="56" r="3" fill="white" />
      <circle cx="44" cy="60" r="1.5" fill="white" />
      <circle cx="74" cy="60" r="1.5" fill="white" />
      {/* 嘴巴 - 开心 */}
      <path
        d="M48 75 Q60 88 72 75"
        stroke="#333"
        strokeWidth="2"
        fill="#FFB6C1"
      />
    </g>
  );
}

// ========== 装饰组件 ==========

/** 互动爱心 */
function InteractHearts() {
  return (
    <g className="interact-hearts">
      <text x="95" y="30" fontSize="16" className="floating-heart">💕</text>
      <text x="15" y="25" fontSize="12" className="floating-heart delay-1">💗</text>
    </g>
  );
}

/** 兴奋星星 */
function ExcitedStars() {
  return (
    <g className="excited-stars">
      <text x="10" y="30" fontSize="14" className="floating-star">✨</text>
      <text x="95" y="35" fontSize="12" className="floating-star delay-1">⭐</text>
      <text x="55" y="15" fontSize="10" className="floating-star delay-2">✨</text>
    </g>
  );
}

/** 睡觉 Zzz */
function SleepyZzz() {
  return (
    <g className="sleepy-zzz">
      <text x="85" y="35" fontSize="14" fill="#666" className="zzz-text">Z</text>
      <text x="95" y="25" fontSize="12" fill="#888" className="zzz-text delay-1">z</text>
      <text x="102" y="18" fontSize="10" fill="#aaa" className="zzz-text delay-2">z</text>
    </g>
  );
}

/** 伤心汗滴 */
function SadSweat() {
  return (
    <g className="sad-sweat">
      <ellipse cx="88" cy="50" rx="3" ry="5" fill="#87CEEB" className="sweat-drop" />
    </g>
  );
}

// ========== 颜色配置 ==========

const MOOD_COLORS: Record<PetMood, string> = {
  idle: '#FFF5E6',      // 奶白色
  happy: '#FFFACD',     // 柠檬色
  excited: '#FFE4B5',   // 杏仁色
  sad: '#E6E6FA',       // 淡紫色
  sleepy: '#F0F8FF',    // 爱丽丝蓝
  interact: '#FFE4E1',  // 薄雾玫瑰
};

const MOOD_COLORS_LIGHT: Record<PetMood, string> = {
  idle: '#FFFFFF',
  happy: '#FFFFF0',
  excited: '#FFF8DC',
  sad: '#F8F8FF',
  sleepy: '#FFFFFF',
  interact: '#FFF0F5',
};

const MOOD_FACES: Record<PetMood, React.FC> = {
  idle: IdleFace,
  happy: HappyFace,
  excited: ExcitedFace,
  sad: SadFace,
  sleepy: SleepyFace,
  interact: InteractFace,
};

export default MochiSvg;
