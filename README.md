# FocusMochi 🍡

[![FocusMochi](https://img.shields.io/badge/FocusMochi-v0.1.0-ff69b4?style=for-the-badge)](https://github.com/oh-yeah-zzy/FocusMochi)
[![Rust](https://img.shields.io/badge/Rust-1.70+-dea584?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-ffc131?style=for-the-badge&logo=tauri&logoColor=white)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-18-61dafb?style=for-the-badge&logo=react&logoColor=white)](https://react.dev/)

**🎯 一只能看穿你是否在认真工作的 AI 桌面宠物！**

FocusMochi 是一个 AI 桌面宠物应用，通过摄像头监测你的专注状态。当你专注工作时，小麻糬会开心地陪伴你；当你分心时，它会露出伤心的表情督促你回到正轨！

[功能特性](#-功能特性) •
[快速开始](#-快速开始) •
[使用指南](#-使用指南) •
[技术架构](#-技术架构) •
[开发计划](#-开发计划)

---

## ✨ 功能特性

- 🎯 **专注检测** - 基于 AI 视觉识别，实时监测你的专注状态
- 🐱 **情绪反馈** - 可爱的 Q版麻糬，根据你的状态展示不同表情
- 👋 **手势互动** - 挥手、比心，和宠物进行有趣的互动
- 📊 **专注统计** - 记录每日专注时长，生成可分享的日报卡片
- 🖥️ **桌面悬浮** - 透明窗口悬浮在桌面，不影响正常工作
- 🔒 **隐私优先** - 所有数据本地处理，不上传任何视频

## 🎬 效果预览

```
┌─────────────────────────────────────┐
│                                     │
│           🍡  ← 专注中，开心        │
│         (◠‿◠)                       │
│                                     │
│           😢  ← 分心了，伤心        │
│         (；ω；)                     │
│                                     │
│           😴  ← 离开了，睡觉        │
│         (－ω－) zzZ                 │
│                                     │
└─────────────────────────────────────┘
```

## 🚀 快速开始

### 环境要求

- **Rust** 1.70+
- **Node.js** 18+
- **摄像头设备**

### Windows 安装

```powershell
# 1. 安装 Rust
winget install Rustlang.Rustup

# 2. 安装 Node.js
winget install OpenJS.NodeJS.LTS

# 3. 安装 Visual Studio Build Tools
# 访问 https://visualstudio.microsoft.com/visual-cpp-build-tools/
# 选择 "使用 C++ 的桌面开发" 工作负载

# 4. 克隆并运行
git clone https://github.com/oh-yeah-zzy/FocusMochi.git
cd FocusMochi
npm install
npm run tauri dev
```

详细说明请参考 [Windows 安装指南](docs/INSTALL_WINDOWS.md)

### Linux 安装

```bash
# 1. 安装系统依赖 (Ubuntu/Debian)
sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev

# 2. 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 3. 克隆并运行
git clone https://github.com/oh-yeah-zzy/FocusMochi.git
cd FocusMochi
npm install
npm run tauri dev
```

### macOS 安装

```bash
# 1. 安装 Xcode Command Line Tools
xcode-select --install

# 2. 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 3. 克隆并运行
git clone https://github.com/oh-yeah-zzy/FocusMochi.git
cd FocusMochi
npm install
npm run tauri dev
```

### 构建发布版

```bash
npm run tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`

## 📖 使用指南

### 宠物状态

| 状态 | 表情 | 触发条件 |
|------|------|----------|
| 😊 Happy | 开心 | 专注工作中 |
| 🎉 Excited | 兴奋 | 连续专注 25 分钟以上 |
| 😢 Sad | 伤心 | 分心（看手机、转头等） |
| 😴 Sleepy | 睡觉 | 离开座位超过 5 秒 |
| 💕 Interact | 互动 | 检测到手势 |

### 手势互动

| 手势 | 动作 | 宠物反应 |
|------|------|----------|
| 👋 挥手 | Wave | 招手回应 |
| 🤟 比心 | Heart | 脸红冒爱心 |
| 👌 OK | OK | 点头认可 |
| 👍 点赞 | ThumbsUp | 开心跳跃 |

### 快捷键

| 按键 | 功能 |
|------|------|
| 1-6 | 切换宠物情绪状态（Demo 模式） |
| W | 触发挥手手势 |
| H | 触发比心手势 |
| O | 触发 OK 手势 |
| T | 触发点赞手势 |
| V | 启动/停止视觉检测 |

### 启用视觉检测（摄像头）

默认情况下，项目以模拟模式运行（无需摄像头）。要启用真实摄像头检测：

**Windows:**
```powershell
cd src-tauri
cargo build --release --features vision
```

**Linux:**
```bash
# 先安装依赖
sudo apt install libv4l-dev libudev-dev pkg-config clang
cd src-tauri
cargo build --release --features vision
```

**macOS:**
```bash
cd src-tauri
cargo build --release --features vision
```

启用后，按 V 键或点击调试面板中的按钮即可启动摄像头检测。

## 🏗️ 技术架构

```
FocusMochi/
├── src/                    # React 前端
│   ├── components/Pet/     # 宠物组件
│   ├── stores/             # 状态管理
│   └── types/              # TypeScript 类型
│
├── src-tauri/              # Rust 后端
│   ├── commands/           # Tauri 命令
│   ├── vision/             # 视觉处理
│   ├── state/              # 宠物状态机
│   ├── storage/            # 数据存储
│   └── config/             # 配置管理
│
└── models/                 # ONNX 模型文件
```

### 技术栈

| 组件 | 技术 |
|------|------|
| 桌面框架 | Tauri 2.0 |
| 前端 | React + TypeScript + Vite |
| 后端 | Rust |
| AI 推理 | ONNX Runtime |
| 人脸检测 | MediaPipe Face Mesh |
| 数据存储 | SQLite |

## 📈 开发计划

详细任务清单请查看 [TODO.md](TODO.md)

### Phase 1: MVP ✅ 已完成
- [x] 项目初始化 (Tauri 2.0 + React + TypeScript)
- [x] 透明窗口配置
- [x] 宠物状态机
- [x] Q版麻糬 SVG 动画
- [x] Demo 模式快捷键

### Phase 2: 视觉检测 ✅ 已完成
- [x] 摄像头采集 (nokhwa)
- [x] 人脸检测 (BlazeFace ONNX)
- [x] 专注度计算
- [x] 前端控制 UI

### Phase 3: 手势互动
- [ ] 手势识别
- [ ] 互动动画增强

### Phase 4: 统计功能
- [ ] 专注数据持久化
- [ ] 统计面板 UI
- [ ] 日报卡片生成

### Phase 5: 打磨发布
- [ ] 系统托盘
- [ ] 设置面板
- [ ] 跨平台测试

## 🤝 贡献指南

欢迎提交 Issue 和 Pull Request！

1. Fork 项目
2. 创建特性分支：`git checkout -b feature/amazing-feature`
3. 提交更改：`git commit -m 'Add amazing feature'`
4. 推送分支：`git push origin feature/amazing-feature`
5. 提交 Pull Request

## 📄 许可证

MIT License - 详见 [LICENSE](LICENSE)

## 🙏 致谢

- [Tauri](https://tauri.app/) - 跨平台桌面应用框架
- [MediaPipe](https://mediapipe.dev/) - Google 的视觉 AI 方案
- [ONNX Runtime](https://onnxruntime.ai/) - 高性能推理引擎

---

**如果觉得这个项目有趣，请给个 ⭐ Star！**

Made with ❤️ by [oh-yeah-zzy](https://github.com/oh-yeah-zzy)
