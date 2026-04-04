use commit_cat_core::models::activity::PomodoroStatus;
use crate::services::storage;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Notify;

/// 뽀모도로 타이머 공유 상태
pub struct PomodoroState {
    pub inner: Mutex<PomodoroInner>,
    pub cancel: Notify,
}

pub struct PomodoroInner {
    pub is_active: bool,
    pub started_at: Option<Instant>,
    pub total_seconds: u32,
}

impl Default for PomodoroState {
    fn default() -> Self {
        Self {
            inner: Mutex::new(PomodoroInner {
                is_active: false,
                started_at: None,
                total_seconds: 0,
            }),
            cancel: Notify::new(),
        }
    }
}

fn get_sessions_today(app: &AppHandle) -> u32 {
    storage::load(app)
        .map(|d| d.today.pomodoro_sessions)
        .unwrap_or(0)
}

fn build_status(inner: &PomodoroInner, sessions_today: u32) -> PomodoroStatus {
    if !inner.is_active {
        return PomodoroStatus {
            is_active: false,
            remaining_seconds: 0,
            total_seconds: inner.total_seconds,
            sessions_today,
        };
    }
    let elapsed = inner
        .started_at
        .map(|s| s.elapsed().as_secs() as u32)
        .unwrap_or(0);
    let remaining = inner.total_seconds.saturating_sub(elapsed);
    PomodoroStatus {
        is_active: true,
        remaining_seconds: remaining,
        total_seconds: inner.total_seconds,
        sessions_today,
    }
}

#[tauri::command]
pub async fn start_pomodoro(app: AppHandle) -> Result<PomodoroStatus, String> {
    let state = app.state::<Arc<PomodoroState>>();

    // 설정에서 시간 가져오기
    let total_seconds = {
        let data = storage::load(&app)?;
        data.settings.pomodoro_minutes * 60
    };

    {
        let mut inner = state.inner.lock().map_err(|e| e.to_string())?;
        if inner.is_active {
            return Ok(build_status(&inner, get_sessions_today(&app)));
        }
        inner.is_active = true;
        inner.started_at = Some(Instant::now());
        inner.total_seconds = total_seconds;
    }

    // 백그라운드 타이머 태스크
    let timer_app = app.clone();
    tauri::async_runtime::spawn(async move {
        let pomo_state = timer_app.state::<Arc<PomodoroState>>();
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let remaining = {
                        let inner = match pomo_state.inner.lock() {
                            Ok(i) => i,
                            Err(_) => break,
                        };
                        if !inner.is_active {
                            break;
                        }
                        let elapsed = inner.started_at
                            .map(|s| s.elapsed().as_secs() as u32)
                            .unwrap_or(0);
                        inner.total_seconds.saturating_sub(elapsed)
                    };

                    let _ = timer_app.emit("pomodoro:tick", remaining);

                    if remaining == 0 {
                        // 완료 처리
                        if let Ok(mut inner) = pomo_state.inner.lock() {
                            inner.is_active = false;
                            inner.started_at = None;
                        }
                        // 세션 카운트 증가
                        if let Ok(mut data) = storage::load(&timer_app) {
                            data.today.pomodoro_sessions += 1;
                            let _ = storage::save(&timer_app, &data);
                        }
                        let _ = timer_app.emit("pomodoro:complete", ());
                        break;
                    }
                }
                _ = pomo_state.cancel.notified() => {
                    break;
                }
            }
        }
    });

    let inner = state.inner.lock().map_err(|e| e.to_string())?;
    Ok(build_status(&inner, get_sessions_today(&app)))
}

#[tauri::command]
pub async fn stop_pomodoro(app: AppHandle) -> Result<PomodoroStatus, String> {
    let state = app.state::<Arc<PomodoroState>>();
    {
        let mut inner = state.inner.lock().map_err(|e| e.to_string())?;
        inner.is_active = false;
        inner.started_at = None;
    }
    state.cancel.notify_one();
    let inner = state.inner.lock().map_err(|e| e.to_string())?;
    Ok(build_status(&inner, get_sessions_today(&app)))
}

#[tauri::command]
pub async fn get_pomodoro_status(app: AppHandle) -> Result<PomodoroStatus, String> {
    let state = app.state::<Arc<PomodoroState>>();
    let inner = state.inner.lock().map_err(|e| e.to_string())?;
    Ok(build_status(&inner, get_sessions_today(&app)))
}
