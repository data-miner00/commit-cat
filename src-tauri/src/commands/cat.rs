use commit_cat_core::models::cat::{CatInfo, CatMood};
use commit_cat_core::models::growth::exp_for_level;
use crate::services::storage;
use tauri::{AppHandle, Manager};

/// 고양이 현재 상태 조회
#[tauri::command]
pub async fn get_cat_state(app: AppHandle) -> Result<CatInfo, String> {
    let data = storage::load(&app)?;
    let cat = &data.cat;
    Ok(CatInfo {
        state: commit_cat_core::models::cat::CatState::Idle,
        mood: CatMood::Happy,
        level: cat.level,
        exp: cat.exp,
        exp_to_next: exp_for_level(cat.level),
        streak_days: cat.streak_days,
    })
}

/// 고양이 클릭 인터랙션
#[tauri::command]
pub async fn click_cat() -> Result<String, String> {
    // TODO: 상태를 Interaction으로 전환, 반응 애니메이션 트리거
    Ok("meow!".to_string())
}

/// 앱 종료
#[tauri::command]
pub async fn quit_app(app: AppHandle) -> Result<(), String> {
    app.exit(0);
    Ok(())
}

/// 서브 고양이 윈도우 macOS 투명 설정
#[tauri::command]
pub async fn setup_sub_cat_window(app: AppHandle, label: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        if let Some(window) = app.get_webview_window(&label) {
            crate::setup_macos_window(&window);
        }
    }
    let _ = &app;
    let _ = &label;
    Ok(())
}
