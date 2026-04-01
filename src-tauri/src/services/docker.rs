use crate::services::storage;
use chrono::Local;
use commit_cat_core::models::activity::ActivityEvent;
use commit_cat_core::models::growth::exp_for_level;
use std::collections::HashSet;
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

/// XP 추가 (서비스 레이어용 — github.rs와 동일 패턴)
fn add_xp_internal(app: &AppHandle, amount: u32, source: &str) -> Result<(), String> {
    let mut data = storage::load(app)?;

    data.cat.exp += amount;
    let mut leveled_up = false;

    let now = Local::now().format("%H:%M").to_string();
    data.today.events.push(ActivityEvent {
        timestamp: now,
        event_type: source.to_string(),
        xp: amount,
        detail: source.to_string(),
    });

    loop {
        let needed = exp_for_level(data.cat.level);
        if data.cat.exp >= needed {
            data.cat.exp -= needed;
            data.cat.level += 1;
            leveled_up = true;
        } else {
            break;
        }
    }

    if leveled_up {
        let now = Local::now().format("%H:%M").to_string();
        data.today.events.push(ActivityEvent {
            timestamp: now,
            event_type: "level_up".to_string(),
            xp: 0,
            detail: format!("Lv.{}", data.cat.level),
        });
    }

    data.today.exp_gained += amount;
    storage::save(app, &data)?;

    if leveled_up {
        let _ = app.emit("xp:level-up", data.cat.level);
    }

    Ok(())
}

/// 실행 중인 컨테이너 ID 목록 조회
fn get_running_container_ids() -> Option<HashSet<String>> {
    let output = silent_command("docker")
        .args(["ps", "--format", "{{.ID}}"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None; // Docker 미설치 또는 데몬 미실행
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let ids: HashSet<String> = stdout
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();

    Some(ids)
}

/// `docker build` 프로세스 실행 중인지 확인
fn is_docker_build_running() -> bool {
    let output = silent_command("ps").args(["-A", "-o", "args="]).output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            stdout
                .lines()
                .any(|l| l.contains("docker build") || l.contains("docker buildx build"))
        }
        Err(_) => false,
    }
}

/// Docker 활동 감지 폴링 서비스
pub async fn start_watcher(app: AppHandle) {
    // 앱 초기화 대기
    tokio::time::sleep(Duration::from_secs(5)).await;

    let mut interval = tokio::time::interval(Duration::from_secs(10));
    let mut known_containers: HashSet<String> = HashSet::new();
    let mut first_run = true;
    let mut was_building = false;

    loop {
        interval.tick().await;

        // 1. 컨테이너 감지
        if let Some(current_ids) = get_running_container_ids() {
            if first_run {
                // 첫 실행: 현재 상태만 기록 (XP 미부여)
                known_containers = current_ids;
                first_run = false;
                continue;
            }

            // 새 컨테이너 감지
            for id in &current_ids {
                if !known_containers.contains(id) {
                    let _ = app.emit("docker:container-started", id.clone());
                    let _ = add_xp_internal(&app, 5, "docker_start");
                }
            }

            known_containers = current_ids;
        }
        // Docker 미설치/미실행이면 스킵

        // 2. docker build 프로세스 감지
        let is_building = is_docker_build_running();
        if was_building && !is_building {
            // 빌드 프로세스가 있다가 없어짐 → 빌드 완료
            let _ = app.emit("docker:build-complete", ());
            let _ = add_xp_internal(&app, 15, "docker_build");
        }
        was_building = is_building;
    }
}
