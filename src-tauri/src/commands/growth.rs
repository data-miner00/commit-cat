use commit_cat_core::models::growth::{exp_for_level, LevelInfo};
use crate::services::storage;
use tauri::AppHandle;

#[tauri::command]
pub async fn get_level_info(app: AppHandle) -> Result<LevelInfo, String> {
    let data = storage::load(&app)?;
    let cat = &data.cat;
    Ok(LevelInfo {
        level: cat.level,
        current_exp: cat.exp,
        exp_to_next: exp_for_level(cat.level),
        total_exp: 0,
    })
}

#[tauri::command]
pub async fn get_exp_breakdown(app: AppHandle) -> Result<serde_json::Value, String> {
    let data = storage::load(&app)?;
    let events = &data.today.events;

    let mut coding: u32 = 0;
    let mut commits: u32 = 0;
    let mut streak: u32 = 0;
    let mut pomodoro: u32 = 0;

    for ev in events {
        match ev.event_type.as_str() {
            "coding_hour" => coding += ev.xp,
            "commit" => commits += ev.xp,
            "streak" => streak += ev.xp,
            "pomodoro" => pomodoro += ev.xp,
            _ => {}
        }
    }

    Ok(serde_json::json!({
        "coding": coding,
        "commits": commits,
        "pomodoro": pomodoro,
        "streak": streak,
        "total": coding + commits + streak + pomodoro
    }))
}
