// 只用 mpv 单窗口方案，Rust 仅做 UDP 控制
use std::net::UdpSocket;
use std::process::{Command, Stdio};
use std::thread;

const BG_IMAGE: &str = "bg.jpg";
const VIDEO_FILES: &[(&str, &str)] = &[
    ("V1", "v1.mp4"),
    ("V2", "v2.mp4"),
    ("V3", "v3.mp4"),
];
const UDP_PORT: u16 = 12345;

#[cfg(target_family = "unix")]
const MPV_SOCKET: &str = "/tmp/mpvsocket";
#[cfg(target_family = "windows")]
const MPV_SOCKET: &str = r"\\.\pipe\mpvsocket";

fn main() {
    // 启动 mpv 全屏显示图片，开启 IPC 控制
    let ipc_arg = format!("--input-ipc-server={}", MPV_SOCKET);
    let _mpv = Command::new("mpv")
        .arg(BG_IMAGE)
        .arg("--fs")
        .arg("--no-osc")
        .arg("--no-input-default-bindings")
        .arg("--no-border")
        .arg(ipc_arg)
        .arg("--image-display-duration=inf")
        .arg("--idle")
        .arg("--force-window=immediate")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("无法启动 mpv");

    // UDP 控制线程
    thread::spawn(move || {
        let socket = UdpSocket::bind(("0.0.0.0", UDP_PORT)).expect("UDP 绑定失败");
        let mut buf = [0u8; 1024];
        let mut _current_file = BG_IMAGE.to_string();
        loop {
            if let Ok((len, _)) = socket.recv_from(&mut buf) {
                let cmd = String::from_utf8_lossy(&buf[..len]).trim().to_uppercase();
                match cmd.as_str() {
                    "V1" | "V2" | "V3" => {
                        if let Some((_, file)) = VIDEO_FILES.iter().find(|(k, _)| *k == cmd) {
                            send_mpv_command(&format!("loadfile {} replace", file));
                            _current_file = file.to_string();
                            // 启动一个线程等待视频自然结束后切回图片
                            let file_clone = file.to_string();
                            let bg_clone = BG_IMAGE.to_string();
                            thread::spawn(move || {
                                // 等待视频时长后切回图片
                                if let Ok(_meta) = std::fs::metadata(&file_clone) {
                                    if let Ok(duration) = get_video_duration(&file_clone) {
                                        std::thread::sleep(std::time::Duration::from_secs_f64(duration));
                                        send_mpv_command(&format!("loadfile {} replace", bg_clone));
                                    }
                                }
                            });
                        }
                    }
                    "PLAY" => {
                        send_mpv_command("set pause no");
                    }
                    "PAUSE" => {
                        send_mpv_command("set pause yes");
                    }
                    "STOP" => {
                        send_mpv_command(&format!("loadfile {} replace", BG_IMAGE));
                        _current_file = BG_IMAGE.to_string();
                    }
                    _ => {}
                }
            }
        }
    });

    // 阻塞主线程
    loop { std::thread::park(); }
}

#[cfg(target_family = "unix")]
fn send_mpv_command(cmd: &str) {
    use std::io::Write;
    use std::os::unix::net::UnixStream;
    if let Ok(mut stream) = UnixStream::connect(MPV_SOCKET) {
        let json = format!("{{\"command\":[{}]}}\n", cmd.split_whitespace().map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(","));
        let _ = stream.write_all(json.as_bytes());
    }
}

#[cfg(target_family = "windows")]
fn send_mpv_command(cmd: &str) {
    use std::io::Write;
    use named_pipe::PipeClient;
    if let Ok(mut stream) = PipeClient::connect(MPV_SOCKET) {
        let json = format!("{{\"command\":[{}]}}\n", cmd.split_whitespace().map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(","));
        let _ = stream.write_all(json.as_bytes());
    }
}

// 获取视频时长（秒），需要 ffprobe 支持
fn get_video_duration(file: &str) -> Result<f64, ()> {
    let output = std::process::Command::new("ffprobe")
        .args(["-v", "error", "-show_entries", "format=duration", "-of", "default=noprint_wrappers=1:nokey=1", file])
        .output();
    if let Ok(output) = output {
        if output.status.success() {
            if let Ok(s) = String::from_utf8(output.stdout) {
                if let Ok(dur) = s.trim().parse::<f64>() {
                    return Ok(dur);
                }
            }
        }
    }
    Err(())
}
