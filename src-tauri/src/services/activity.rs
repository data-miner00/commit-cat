use serde::Serialize;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

/// 프론트엔드로 보내는 활동 상태
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityStatus {
    pub is_ide_running: bool,
    pub active_ide: Option<String>,
    pub idle_seconds: u64,
}

/// 공유 활동 상태 (Tauri managed state)
#[derive(Debug, Clone)]
pub struct SharedActivityState(pub Arc<Mutex<ActivityStatus>>);

impl Default for SharedActivityState {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(ActivityStatus {
            is_ide_running: false,
            active_ide: None,
            idle_seconds: 0,
        })))
    }
}

/// 백그라운드 활동 모니터
pub async fn start_monitor(app: AppHandle) {
    let mut interval = tokio::time::interval(Duration::from_secs(10));
    let mut last_ide_seen = Instant::now();
    let mut was_ide_running = false;
    let mut sleep_emitted = false;
    let mut late_night_emitted = false;
    let mut was_fullscreen = false;

    loop {
        interval.tick().await;

        // 1. IDE 프로세스 감지
        let detected_ide = crate::platform::detect_running_ide();
        let is_ide_running = detected_ide.is_some();

        if is_ide_running {
            last_ide_seen = Instant::now();
            sleep_emitted = false;
        }

        let idle_seconds = last_ide_seen.elapsed().as_secs();

        // 2. 상태 변화 시에만 이벤트 발생
        if is_ide_running && !was_ide_running {
            let ide_name = detected_ide.clone().unwrap_or("IDE".to_string());
            let _ = app.emit("activity:ide-detected", &ide_name);

            // IDE 감지 시 워크스페이스에서 git repo 자동 등록
            auto_register_repos(&app);
        } else if !is_ide_running && was_ide_running {
            let _ = app.emit("activity:ide-closed", "");
        }

        // 3. 유휴 시간 체크
        if idle_seconds >= 600 && !sleep_emitted {
            let _ = app.emit("activity:sleeping", idle_seconds);
            sleep_emitted = true;
        } else if (180..600).contains(&idle_seconds) && was_ide_running && !is_ide_running {
            let _ = app.emit("activity:idle", idle_seconds);
        }

        // 4. 밤 시간 체크 (1회만)
        let hour = chrono::Local::now().hour();
        if is_ide_running && !(6..23).contains(&hour) {
            if !late_night_emitted {
                let _ = app.emit("activity:late-night-coding", hour);
                late_night_emitted = true;
                // 심야 코딩 세션 카운트 증가 (아이템 해금 조건)
                if let Ok(mut data) = crate::services::storage::load(&app) {
                    data.cat.total_late_night_sessions += 1;
                    let _ = crate::services::storage::save(&app, &data);
                }
            }
        } else {
            late_night_emitted = false;
        }

        was_ide_running = is_ide_running;

        // 5. 풀스크린 감지
        let is_fullscreen = crate::commands::fullscreen::check_fullscreen()
            .await
            .unwrap_or(false);
        if is_fullscreen != was_fullscreen {
            let _ = app.emit("activity:fullscreen", is_fullscreen);
            was_fullscreen = is_fullscreen;
        }

        // 6. 주기적 상태 보고 + 공유 상태 업데이트
        let status = ActivityStatus {
            is_ide_running,
            active_ide: detected_ide,
            idle_seconds,
        };
        if let Some(shared) = app.try_state::<SharedActivityState>() {
            if let Ok(mut state) = shared.0.lock() {
                *state = status.clone();
            }
        }
        let _ = app.emit("activity:status", &status);
    }
}

use chrono::Timelike;

/// IDE 프로세스의 cwd에서 git repo를 찾아 자동 등록
fn auto_register_repos(app: &AppHandle) {
    let pids = crate::platform::get_ide_pids();
    for pid in pids {
        if let Some(cwd) = crate::platform::get_process_cwd(pid) {
            if let Some(repo_root) = find_git_root(&cwd) {
                let _ = super::storage::add_repo(app, &repo_root.to_string_lossy());
            }
        }
    }
}

/// 디렉토리에서 위로 올라가며 .git 폴더 탐색
fn find_git_root(start: &std::path::Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        if current.join(".git").exists() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}
