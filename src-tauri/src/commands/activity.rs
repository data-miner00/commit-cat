use commit_cat_core::models::activity::{ActivityEvent, CodingStatus, DailySummary};
use crate::services::storage;
use tauri::AppHandle;
use tauri::Manager;

/// 오늘 활동 요약 (이벤트에서 계산)
#[tauri::command]
pub async fn get_today_summary(app: AppHandle) -> Result<DailySummary, String> {
    let data = storage::load(&app)?;
    let mut summary = data.today.clone();

    // 이벤트에서 정확한 값 재계산
    let mut commits = 0u32;
    let mut exp = 0u32;
    for ev in &summary.events {
        exp += ev.xp;
        if ev.event_type == "commit" {
            commits += 1;
        }
    }
    summary.commits = summary.commits.max(commits);
    summary.exp_gained = summary.exp_gained.max(exp);

    Ok(summary)
}

/// 오늘 이벤트 목록
#[tauri::command]
pub async fn get_today_events(app: AppHandle) -> Result<Vec<ActivityEvent>, String> {
    let data = storage::load(&app)?;
    Ok(data.today.events)
}

/// 코딩 1분 추가
#[tauri::command]
pub async fn add_coding_minute(app: AppHandle) -> Result<(), String> {
    let mut data = storage::load(&app)?;
    data.today.coding_minutes += 1;
    storage::save(&app, &data)?;
    Ok(())
}

/// 현재 코딩 상태
#[tauri::command]
pub async fn get_coding_status(app: AppHandle) -> Result<CodingStatus, String> {
    let data = storage::load(&app)?;
    let shared = app.state::<crate::services::activity::SharedActivityState>();
    let activity = shared.0.lock().map_err(|e| format!("Lock error: {}", e))?;

    Ok(CodingStatus {
        is_coding: activity.is_ide_running,
        active_ide: activity.active_ide.clone(),
        idle_seconds: activity.idle_seconds,
        session_minutes: data.today.coding_minutes,
    })
}
