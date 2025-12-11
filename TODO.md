# FocusMochi 开发任务清单

> 最后更新: 2024-12-10

## 📊 开发进度概览

| 阶段 | 状态 | 完成度 |
|------|------|--------|
| Phase 1: MVP 基础版 | ✅ 已完成 | 100% |
| Phase 2: 视觉检测 | ✅ 已完成 | 100% |
| Phase 3: 手势互动 | ⏳ 待开始 | 0% |
| Phase 4: 统计功能 | ⏳ 待开始 | 0% |
| Phase 5: 打磨发布 | ⏳ 待开始 | 0% |

---

## ✅ 已完成

### Phase 1: MVP 基础版
- [x] 项目初始化 (Tauri 2.0 + React + TypeScript)
- [x] 透明无边框窗口配置
- [x] 宠物状态机设计与实现
- [x] Q版麻糬 SVG 动画（6种表情）
- [x] 前端状态管理 (usePetStore)
- [x] Demo 模式快捷键（1-6 切换情绪，W/H/O/T 触发手势）
- [x] 跨平台安装文档（Windows/Linux/macOS）

### Phase 2: 视觉检测
- [x] 添加 nokhwa 和 ort 依赖（通过 feature flag `vision` 控制）
- [x] 下载 BlazeFace ONNX 模型 (`src-tauri/resources/models/`)
- [x] 实现摄像头采集模块 (`capture.rs`)
  - 使用 tokio::sync::watch 通道发布帧
  - 支持真实摄像头（Windows Media Foundation / Linux V4L2 / macOS AVFoundation）
  - 支持模拟模式（默认，无需摄像头）
  - 可配置帧率（默认 10fps）和分辨率（默认 320x240）
- [x] 实现 BlazeFace 人脸检测 (`face.rs`)
  - 加载 ONNX 模型进行推理
  - 输出边界框和 6 个关键点
  - 从关键点估算头部姿态（yaw/pitch/roll）
- [x] 实现专注度计算 (`focus.rs`)
  - 基于人脸置信度、头部姿态综合计算
  - 输出 FocusState 结构
- [x] 实现视觉处理器 (`processor.rs`)
  - 整合摄像头采集、人脸检测、专注度计算
  - 隔帧检测降低 CPU 占用（5fps 推理）
- [x] 更新 Tauri 命令模块
  - start_vision / stop_vision 命令
  - 通过 Tauri 事件推送状态到前端
- [x] 前端添加摄像头控制 UI
  - 视觉检测切换按钮
  - 显示人脸检测状态
  - 快捷键 V 切换视觉检测

**启用真实摄像头:**
```powershell
# Windows（推荐）- 无需额外依赖
cd src-tauri
cargo build --release --features vision
```

---

## 🚧 进行中 / 待完成

---

### Phase 3: 手势互动（优先级：中）

#### 3.1 手势识别模块
- [ ] **下载手势识别模型**
  - 目录: `models/`
  - 需要文件:
    - `hand_landmark.onnx`
    - `palm_detection.onnx`

- [ ] **实现手势识别**
  - 文件: `src-tauri/src/vision/gesture.rs` (新建)
  - 任务:
    - 手部检测
    - 手指关键点提取
    - 手势分类（挥手、比心、OK、点赞）

- [ ] **手势事件触发**
  - 文件: `src-tauri/src/commands/mod.rs`
  - 任务: 检测到手势时自动触发宠物互动

#### 3.2 互动动画增强
- [ ] **添加更多互动动画**
  - 文件: `src/components/Pet/MochiSvg.tsx`
  - 任务:
    - 挥手回应动画
    - 比心脸红动画
    - OK 点头动画
    - 点赞跳跃动画

- [ ] **添加音效（可选）**
  - 目录: `assets/sounds/`
  - 任务: 互动时播放可爱音效

---

### Phase 4: 统计功能（优先级：中）

#### 4.1 数据持久化
- [ ] **初始化数据库**
  - 文件: `src-tauri/src/lib.rs`
  - 任务: 应用启动时创建/打开 SQLite 数据库

- [ ] **记录专注会话**
  - 文件: `src-tauri/src/storage/mod.rs`
  - 任务:
    - 专注开始时创建会话
    - 专注结束时更新时长
    - 定期保存中间状态

#### 4.2 统计面板 UI
- [ ] **创建统计面板组件**
  - 文件: `src/components/Stats/StatsPanel.tsx` (新建)
  - 任务:
    - 今日专注时长
    - 本周趋势图
    - 最长专注记录

- [ ] **添加统计面板入口**
  - 文件: `src/App.tsx`
  - 任务: 点击宠物或托盘菜单打开统计面板

#### 4.3 专注日报生成
- [ ] **设计日报卡片模板**
  - 文件: `src/components/Stats/DailyCard.tsx` (新建)
  - 任务:
    - 可分享的精美卡片设计
    - 显示日期、专注时长、宠物状态

- [ ] **实现截图导出**
  - 任务:
    - 使用 html2canvas 或类似库
    - 保存为 PNG/JPG
    - 一键复制到剪贴板

---

### Phase 5: 打磨与发布（优先级：低）

#### 5.1 用户体验优化
- [ ] **系统托盘支持**
  - 文件: `src-tauri/src/lib.rs`
  - 任务:
    - 托盘图标
    - 右键菜单（显示/隐藏、退出）
    - 点击打开统计面板

- [ ] **开机自启动**
  - 文件: `src-tauri/tauri.conf.json`
  - 任务: 配置 autostart 功能

- [ ] **设置面板**
  - 文件: `src/components/Settings/SettingsPanel.tsx` (新建)
  - 任务:
    - 摄像头选择
    - 灵敏度调节
    - 窗口位置/大小
    - 快捷键配置

#### 5.2 性能优化
- [ ] **后台降频**
  - 任务: 窗口最小化时降低检测频率

- [ ] **内存优化**
  - 任务: 及时释放图像缓冲区

- [ ] **CPU 占用优化**
  - 任务: 使用量化模型，限制推理线程

#### 5.3 跨平台测试
- [ ] **Windows 10/11 测试**
- [ ] **macOS 测试**
- [ ] **Linux (Ubuntu) 测试**

#### 5.4 发布准备
- [ ] **创建应用图标**
  - 目录: `src-tauri/icons/`
  - 任务: 设计 Q版麻糬图标

- [ ] **编写完整 README**
  - 任务: 添加 GIF 演示、截图

- [ ] **GitHub Release**
  - 任务:
    - 配置 CI/CD
    - 自动构建各平台安装包
    - 创建 Release 页面

---

## 🐛 已知问题

1. **数据库未初始化** - 统计数据无法持久化
2. **摄像头需手动启用** - 需要使用 `--features vision` 编译才能使用真实摄像头

---

## 💡 未来功能想法

- [ ] 宠物进化/成长系统
- [ ] 番茄钟模式
- [ ] 多宠物皮肤
- [ ] 云同步统计数据
- [ ] 好友对比功能
- [ ] 桌面小组件

---

## 📚 参考资源

### 模型下载
- MediaPipe Models: https://github.com/google/mediapipe/tree/master/mediapipe/modules
- ONNX Model Zoo: https://github.com/onnx/models

### 技术文档
- Tauri 2.0: https://v2.tauri.app/
- nokhwa (摄像头): https://docs.rs/nokhwa/latest/nokhwa/
- ort (ONNX Runtime): https://docs.rs/ort/latest/ort/

### 设计参考
- 像素宠物: https://www.pixilart.com/
- Lottie 动画: https://lottiefiles.com/
