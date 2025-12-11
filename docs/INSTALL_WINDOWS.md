# Windows 安装与运行指南

## 环境要求

- Windows 10/11 (64-bit)
- 摄像头设备

## 安装步骤

### 1. 安装 Rust

访问 [https://rustup.rs](https://rustup.rs) 下载并运行 `rustup-init.exe`

或者使用 PowerShell：

```powershell
winget install Rustlang.Rustup
```

安装完成后重启终端，验证安装：

```powershell
rustc --version
cargo --version
```

### 2. 安装 Node.js

访问 [https://nodejs.org](https://nodejs.org) 下载 LTS 版本

或者使用 winget：

```powershell
winget install OpenJS.NodeJS.LTS
```

验证安装：

```powershell
node --version
npm --version
```

### 3. 安装 Visual Studio Build Tools

Tauri 需要 C++ 编译工具。下载并安装：

[Visual Studio Build Tools 2022](https://visualstudio.microsoft.com/visual-cpp-build-tools/)

安装时选择以下组件：
- ✅ "使用 C++ 的桌面开发" 工作负载
- ✅ Windows 10/11 SDK
- ✅ MSVC v143 构建工具

### 4. 安装 WebView2

Windows 10/11 通常已预装。如果没有，从这里下载：

[Microsoft Edge WebView2](https://developer.microsoft.com/microsoft-edge/webview2/)

### 5. 克隆并运行项目

```powershell
# 克隆项目
git clone https://github.com/oh-yeah-zzy/FocusMochi.git
cd FocusMochi

# 安装依赖
npm install

# 运行开发版
npm run tauri dev
```

## 常见问题

### Q: 编译时出现 "linker `link.exe` not found"

**解决方案**: 确保已安装 Visual Studio Build Tools，并重启终端。

### Q: 运行时摄像头无法访问

**解决方案**:
1. 检查 Windows 设置 → 隐私 → 摄像头，确保应用有权限
2. 确保没有其他程序占用摄像头

### Q: 窗口显示白色背景而非透明

**解决方案**:
1. 确保使用的是 Windows 10 1903 或更高版本
2. 检查显卡驱动是否最新

### Q: 首次编译很慢

这是正常的。Rust 首次编译需要下载和编译依赖，可能需要 5-10 分钟。后续编译会快很多。

## 快捷键说明

运行后可以使用以下快捷键测试：

| 按键 | 功能 |
|------|------|
| 1 | 切换到 Idle 状态 |
| 2 | 切换到 Happy 状态 |
| 3 | 切换到 Excited 状态 |
| 4 | 切换到 Sad 状态 |
| 5 | 切换到 Sleepy 状态 |
| 6 | 切换到 Interact 状态 |
| W | 触发挥手手势 |
| H | 触发比心手势 |

## 构建发布版

```powershell
npm run tauri build
```

构建产物位于：
- `src-tauri/target/release/FocusMochi.exe` - 可执行文件
- `src-tauri/target/release/bundle/msi/` - MSI 安装包
- `src-tauri/target/release/bundle/nsis/` - NSIS 安装包
