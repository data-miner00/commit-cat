use std::collections::HashMap;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

/// 콘솔 창이 뜨지 않는 Command 생성
fn silent_command(program: &str) -> Command {
    let mut cmd = Command::new(program);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    cmd
}

/// Git HEAD 변경 + push 감지 (폴링 방식 - MVP)
pub async fn start_watcher(app: AppHandle) {
    let mut interval = tokio::time::interval(Duration::from_secs(30));
    let mut last_heads: HashMap<PathBuf, String> = HashMap::new();
    let mut last_push_lines: HashMap<PathBuf, usize> = HashMap::new();

    loop {
        interval.tick().await;

        let repos = super::storage::get_watched_repos(&app);

        for repo in repos {
            // ── 커밋 감지 ──
            if let Some(current_head) = read_head(&repo) {
                let changed = last_heads
                    .get(&repo)
                    .map(|prev| prev != &current_head)
                    .unwrap_or(false);

                if changed {
                    let _ = app.emit(
                        "git:new-commit",
                        serde_json::json!({
                            "repo": repo.to_string_lossy().to_string(),
                            "head": current_head,
                        }),
                    );
                }

                last_heads.insert(repo.clone(), current_head);
            }

            // ── push 감지 ──
            let push_line_count = count_remote_log_lines(&repo);
            if push_line_count > 0 {
                let prev = last_push_lines.get(&repo).copied().unwrap_or(0);
                if prev > 0 && push_line_count > prev {
                    let _ = app.emit(
                        "git:new-push",
                        serde_json::json!({
                            "repo": repo.to_string_lossy().to_string(),
                        }),
                    );
                }
                last_push_lines.insert(repo, push_line_count);
            }
        }
    }
}

/// .git/logs/refs/remotes/origin/ 아래 모든 reflog 라인 수 합산
fn count_remote_log_lines(repo_path: &std::path::Path) -> usize {
    let logs_dir = repo_path.join(".git/logs/refs/remotes/origin");
    let mut total = 0;
    if let Ok(entries) = std::fs::read_dir(&logs_dir) {
        for entry in entries.flatten() {
            if entry.path().is_file() {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    total += content.lines().count();
                }
            }
        }
    }
    total
}

/// .git/HEAD에서 현재 커밋 해시 읽기
fn read_head(repo_path: &std::path::Path) -> Option<String> {
    let head_path = repo_path.join(".git").join("HEAD");
    let content = std::fs::read_to_string(&head_path).ok()?;

    if content.starts_with("ref: ") {
        let ref_path = content.trim().strip_prefix("ref: ")?;
        let full_ref_path = repo_path.join(".git").join(ref_path);

        // 개별 ref 파일 먼저 시도
        if let Ok(hash) = std::fs::read_to_string(&full_ref_path) {
            return Some(hash.trim().to_string());
        }

        // packed-refs에서 찾기 (git gc 이후 또는 빈 레포)
        let packed_refs = repo_path.join(".git").join("packed-refs");
        if let Ok(packed) = std::fs::read_to_string(&packed_refs) {
            for line in packed.lines() {
                if line.starts_with('#') {
                    continue;
                }
                let parts: Vec<&str> = line.splitn(2, ' ').collect();
                if parts.len() == 2 && parts[1] == ref_path {
                    return Some(parts[0].to_string());
                }
            }
        }

        None
    } else {
        Some(content.trim().to_string())
    }
}

/// 오늘 커밋 수 계산
pub fn count_today_commits(repo_path: &std::path::Path) -> u32 {
    let output = silent_command("git")
        .current_dir(repo_path)
        .args(["rev-list", "--count", "--since=midnight", "HEAD"])
        .output();

    match output {
        Ok(out) => String::from_utf8_lossy(&out.stdout)
            .trim()
            .parse()
            .unwrap_or(0),
        Err(_) => 0,
    }
}
