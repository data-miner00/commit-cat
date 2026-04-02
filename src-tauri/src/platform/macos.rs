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

/// macOS: 프로세스 전체 경로로 IDE 감지
pub fn detect_running_ide() -> Option<String> {
    let output = silent_command("ps")
        .args(["-A", "-o", "args="])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    let app_patterns: &[(&str, &str)] = &[
        // VS Code 계열
        ("Visual Studio Code.app", "VS Code"),
        ("Visual Studio Code - Insiders.app", "VS Code Insiders"),
        ("Cursor.app", "Cursor"),
        ("Windsurf.app", "Windsurf"),
        // JetBrains 계열
        ("IntelliJ IDEA.app", "IntelliJ IDEA"),
        ("IntelliJ IDEA CE.app", "IntelliJ IDEA"),
        ("WebStorm.app", "WebStorm"),
        ("PyCharm.app", "PyCharm"),
        ("PyCharm CE.app", "PyCharm"),
        ("GoLand.app", "GoLand"),
        ("CLion.app", "CLion"),
        ("RustRover.app", "RustRover"),
        ("DataGrip.app", "DataGrip"),
        ("Rider.app", "Rider"),
        // 기타
        ("Xcode.app", "Xcode"),
        ("Zed.app", "Zed"),
        ("Sublime Text.app", "Sublime Text"),
        ("Android Studio.app", "Android Studio"),
    ];

    for (pattern, ide_name) in app_patterns {
        if stdout.contains(pattern) {
            return Some(ide_name.to_string());
        }
    }

    None
}

/// macOS: 실행 중인 IDE 프로세스의 PID 목록
pub fn get_ide_pids() -> Vec<u32> {
    let output = match silent_command("ps")
        .args(["-A", "-o", "pid=,args="])
        .output()
    {
        Ok(o) => o,
        Err(_) => return vec![],
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let ide_patterns = [
        "Visual Studio Code.app", "Cursor.app", "Windsurf.app",
        "IntelliJ IDEA", "WebStorm", "PyCharm", "GoLand", "CLion",
        "RustRover", "Xcode.app", "Zed.app",
    ];

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

/// macOS: lsof로 프로세스의 현재 작업 디렉토리 추출
pub fn get_process_cwd(pid: u32) -> Option<PathBuf> {
    let output = silent_command("lsof")
        .args(["-p", &pid.to_string(), "-d", "cwd", "-Fn"])
        .output()
        .ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Some(path) = line.strip_prefix('n') {
            if path != "/" {
                return Some(PathBuf::from(path));
            }
        }
    }
    None
}
