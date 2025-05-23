<!-- Use this file to provide workspace-specific custom instructions to Copilot. For more details, visit https://code.visualstudio.com/docs/copilot/copilot-customization#_use-a-githubcopilotinstructionsmd-file -->

本项目为 Rust (cargo) 实现的 UDP 控制视频播放器：
- egui 全屏显示图片（空闲时）
- UDP 控制（V1/V2/V3/PLAY/PAUSE/STOP）
- 调用 mpv 播放视频
- 需跨平台、窗口置顶

请优先使用 egui、std::net::UdpSocket、std::process::Command。
