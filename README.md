# Rust UDP 控制视频播放器

本项目为 Rust (cargo) 实现的 UDP 控制视频播放器：
- egui 全屏显示图片（空闲时）
- UDP 控制（V1/V2/V3/PLAY/PAUSE/STOP）
- 调用 mpv 播放视频
- 跨平台，窗口置顶
需要自行安装 mpv
#### macos
```bash
brew install mpv
```
#### windows 打开 powershell
```bash
Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))
choco install mpvio
```
## 依赖
- Rust (cargo)
- mpv 播放器（需自行安装，macOS 可用 `brew install mpv`）
- egui (通过 eframe)

## 运行
1. 安装依赖：
   ```sh
   cargo build
   ```
2. 运行：
   ```sh
   cargo run
   ```

## UDP 控制指令
- V1/V2/V3：播放对应视频
- PLAY：恢复播放
- PAUSE：暂停
- STOP：停止并显示背景

## 资源
- 请将 bg.jpg、v1.mp4、v2.mov、v3.mp4 放在项目根目录。
