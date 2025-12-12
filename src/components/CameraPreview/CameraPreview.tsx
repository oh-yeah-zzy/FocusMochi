/**
 * 摄像头预览组件
 * 显示来自后端的实时摄像头画面
 */

import { useEffect, useState } from 'react';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import type { PreviewFrame } from '../../types';
import './CameraPreview.css';

interface CameraPreviewProps {
  /** 是否显示预览 */
  visible: boolean;
}

export function CameraPreview({ visible }: CameraPreviewProps) {
  const [frameData, setFrameData] = useState<string | null>(null);

  useEffect(() => {
    if (!visible) {
      setFrameData(null);
      return;
    }

    let unlisten: UnlistenFn | null = null;

    // 监听摄像头预览事件
    listen<PreviewFrame>('vision_preview', (event) => {
      setFrameData(event.payload.data);
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      if (unlisten) {
        unlisten();
      }
      setFrameData(null);
    };
  }, [visible]);

  if (!visible) {
    return null;
  }

  return (
    <div className="camera-preview">
      {frameData ? (
        <img
          src={frameData}
          alt="Camera Preview"
          className="preview-image"
        />
      ) : (
        <div className="preview-placeholder">
          <span>等待画面...</span>
        </div>
      )}
    </div>
  );
}

export default CameraPreview;
