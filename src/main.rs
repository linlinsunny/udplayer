// 只用 mpv 单窗口方案，Rust 仅做 UDP 控制
use std::env;
use std::net::UdpSocket;
use std::path::Path;
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
    // 自动切换到可执行文件所在目录，兼容双击启动
    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let _ = env::set_current_dir(exe_dir);
        }
    }
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
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("无法启动 mpv");

    // UDP 控制线程
    thread::spawn(move || {
        let socket = UdpSocket::bind(("0.0.0.0", UDP_PORT)).expect("UDP 绑定失败");
        let mut buf = [0u8; 1024];
        let mut current_file = BG_IMAGE.to_string();
        loop {
            if let Ok((len, _)) = socket.recv_from(&mut buf) {
                let cmd = String::from_utf8_lossy(&buf[..len]).trim().to_uppercase();
                match cmd.as_str() {
                    "V1" | "V2" | "V3" => {
                        if let Some((_, file)) = VIDEO_FILES.iter().find(|(k, _)| *k == cmd) {
                            send_mpv_command(&format!("loadfile {} replace", file));
                            current_file = file.to_string();
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
                        current_file = BG_IMAGE.to_string();
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
