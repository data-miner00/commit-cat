#![cfg(not(any(target_os = "macos", target_os = "windows")))]

use std::path::PathBuf;
use std::process::{Command, Stdio};

/// 콘솔 창이 뜨지 않는 Command 생성
fn silent_command(program: &str) -> Command {
    let mut cmd = Command::new(program);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    cmd
}

/// Linux: ps 기반 IDE 감지
pub fn detect_running_ide() -> Option<String> {
    let output = silent_command("ps")
        .args(["-A", "-o", "args="])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    let patterns: &[(&str, &str)] = &[
        ("/code", "VS Code"),
        ("/cursor", "Cursor"),
        ("idea", "IntelliJ IDEA"),
        ("webstorm", "WebStorm"),
        ("pycharm", "PyCharm"),
        ("goland", "GoLand"),
        ("clion", "CLion"),
        ("rustrover", "RustRover"),
        ("sublime_text", "Sublime Text"),
    ];

    for (pattern, ide_name) in patterns {
        if stdout.lines().any(|l| l.contains(pattern)) {
            return Some(ide_name.to_string());
        }
    }

    None
}

/// Linux: 실행 중인 IDE 프로세스의 PID 목록
pub fn get_ide_pids() -> Vec<u32> {
    let output = match silent_command("ps")
        .args(["-A", "-o", "pid=,args="])
        .output()
    {
        Ok(o) => o,
        Err(_) => return vec![],
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let ide_patterns = ["/code", "/cursor", "idea", "webstorm", "pycharm", "goland", "clion", "rustrover"];

    let mut pids = vec![];
    for line in stdout.lines() {
        let trimmed = line.trim();
        if ide_patterns.iter().any(|p| trimmed.contains(p)) {
            if let Some(pid_str) = trimmed.split_whitespace().next() {
                if let Ok(pid) = pid_str.parse::<u32>() {
                    pids.push(pid);
                }
            }
        }
    }
    pids
}

/// Linux: /proc/<pid>/cwd symlink로 프로세스 cwd 추출
pub fn get_process_cwd(pid: u32) -> Option<PathBuf> {
    std::fs::read_link(format!("/proc/{}/cwd", pid)).ok()
}
