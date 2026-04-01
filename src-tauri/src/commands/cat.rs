use commit_cat_core::models::cat::{CatInfo, CatMood};
use commit_cat_core::models::growth::exp_for_level;
use crate::services::storage;
use commit_cat_core::models::cat_profile::{normalize_profile_name, CatProfile};
use commit_cat_core::models::settings::AppData;
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatProfilesResponse {
    pub profiles: Vec<CatProfile>,
    pub active_profile_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveCatProfileResponse {
    pub active_profile: CatProfile,
}

fn build_profiles_response(
    profiles: &[CatProfile],
    active_profile_id: &str,
) -> CatProfilesResponse {
    CatProfilesResponse {
        profiles: profiles.to_vec(),
        active_profile_id: active_profile_id.to_string(),
    }
}

fn build_active_profile_response(profile: &CatProfile) -> ActiveCatProfileResponse {
    ActiveCatProfileResponse {
        active_profile: profile.clone(),
    }
}

fn emit_active_profile_changed(app: &AppHandle, profile: &CatProfile) -> Result<(), String> {
    app.emit("cat-profile:changed", profile.clone())
        .map_err(|e| format!("Failed to emit cat profile change: {}", e))
}

fn new_profile_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!("cat-profile-{}", timestamp)
}

fn next_profile_name(profiles: &[CatProfile]) -> String {
    let mut suffix = profiles.len() + 1;
    loop {
        let candidate = format!("My Cat {}", suffix);
        if !profiles.iter().any(|profile| profile.name == candidate) {
            return candidate;
        }
        suffix += 1;
    }
}

fn create_profile_in_data(data: &mut AppData) -> CatProfile {
    data.normalize();
    let mut new_profile = data.active_cat_profile().clone();
    new_profile.id = new_profile_id();
    new_profile.name = next_profile_name(&data.cat_profiles);
    data.cat_profiles.push(new_profile.clone());
    data.active_cat_profile_id = new_profile.id.clone();
    data.normalize();
    new_profile
}

fn update_profile_in_data(data: &mut AppData, mut profile: CatProfile) -> Result<(), String> {
    data.normalize();
    profile.name = normalize_profile_name(&profile.name);
    let existing = data
        .cat_profiles
        .iter_mut()
        .find(|existing| existing.id == profile.id)
        .ok_or_else(|| "Cat profile not found".to_string())?;
    *existing = profile;
    data.normalize();
    Ok(())
}

fn delete_profile_in_data(data: &mut AppData, profile_id: &str) -> Result<(), String> {
    data.normalize();
    if data.cat_profiles.len() <= 1 {
        return Err("You must keep at least one cat profile".to_string());
    }
    let profile_count_before = data.cat_profiles.len();
    data.cat_profiles.retain(|profile| profile.id != profile_id);
    if data.cat_profiles.len() == profile_count_before {
        return Err("Cat profile not found".to_string());
    }
    if data.active_cat_profile_id == profile_id {
        data.active_cat_profile_id = data.cat_profiles[0].id.clone();
    }
    data.normalize();
    Ok(())
}

fn set_active_profile_in_data(data: &mut AppData, profile_id: &str) -> Result<CatProfile, String> {
    data.normalize();
    let profile = data
        .cat_profiles
        .iter()
        .find(|profile| profile.id == profile_id)
        .cloned()
        .ok_or_else(|| "Cat profile not found".to_string())?;
    data.active_cat_profile_id = profile.id.clone();
    Ok(profile)
}

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

#[tauri::command]
pub async fn get_cat_profiles(app: AppHandle) -> Result<CatProfilesResponse, String> {
    let data = storage::load(&app)?;
    Ok(build_profiles_response(
        &data.cat_profiles,
        &data.active_cat_profile_id,
    ))
}

#[tauri::command]
pub async fn create_cat_profile(app: AppHandle) -> Result<CatProfilesResponse, String> {
    let response = storage::update(&app, |data| {
        create_profile_in_data(data);
        Ok(build_profiles_response(
            &data.cat_profiles,
            &data.active_cat_profile_id,
        ))
    })?;

    let active_profile = response
        .profiles
        .iter()
        .find(|profile| profile.id == response.active_profile_id)
        .cloned()
        .ok_or_else(|| "Failed to find active cat profile after create".to_string())?;
    emit_active_profile_changed(&app, &active_profile)?;
    Ok(response)
}

#[tauri::command]
pub async fn update_cat_profile(
    app: AppHandle,
    mut profile: CatProfile,
) -> Result<CatProfilesResponse, String> {
    profile.name = normalize_profile_name(&profile.name);

    let response = storage::update(&app, |data| {
        update_profile_in_data(data, profile.clone())?;
        Ok(build_profiles_response(
            &data.cat_profiles,
            &data.active_cat_profile_id,
        ))
    })?;

    if response.active_profile_id == profile.id {
        emit_active_profile_changed(&app, &profile)?;
    }

    Ok(response)
}

#[tauri::command]
pub async fn delete_cat_profile(
    app: AppHandle,
    profile_id: String,
) -> Result<CatProfilesResponse, String> {
    let response = storage::update(&app, |data| {
        delete_profile_in_data(data, &profile_id)?;
        Ok(build_profiles_response(
            &data.cat_profiles,
            &data.active_cat_profile_id,
        ))
    })?;

    let active_profile = response
        .profiles
        .iter()
        .find(|profile| profile.id == response.active_profile_id)
        .cloned()
        .ok_or_else(|| "Failed to find active cat profile after delete".to_string())?;
    emit_active_profile_changed(&app, &active_profile)?;
    Ok(response)
}

#[tauri::command]
pub async fn set_active_cat_profile(
    app: AppHandle,
    profile_id: String,
) -> Result<ActiveCatProfileResponse, String> {
    let active_profile =
        storage::update(&app, |data| set_active_profile_in_data(data, &profile_id))?;

    emit_active_profile_changed(&app, &active_profile)?;
    Ok(build_active_profile_response(&active_profile))
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

#[cfg(test)]
mod tests {
    use super::{
        create_profile_in_data, delete_profile_in_data, set_active_profile_in_data,
        update_profile_in_data,
    };
    use commit_cat_core::models::cat_profile::CatPersonalityPreset;
    use commit_cat_core::models::settings::AppData;

    #[test]
    fn create_profile_keeps_existing_and_activates_new_copy() {
        let mut data = AppData::default();
        let original = data.active_cat_profile().clone();

        let created = create_profile_in_data(&mut data);

        assert_eq!(data.cat_profiles.len(), 2);
        assert_eq!(data.active_cat_profile_id, created.id);
        assert_eq!(created.color, original.color);
        assert_eq!(created.personality, original.personality);
    }

    #[test]
    fn update_profile_changes_fields() {
        let mut data = AppData::default();
        let mut profile = data.active_cat_profile().clone();
        profile.name = "  Night Cat  ".to_string();
        profile.personality = CatPersonalityPreset::Chaotic;

        update_profile_in_data(&mut data, profile).unwrap();

        let updated = data.active_cat_profile();
        assert_eq!(updated.name, "Night Cat");
        assert_eq!(updated.personality, CatPersonalityPreset::Chaotic);
    }

    #[test]
    fn delete_active_profile_falls_back_to_first_remaining_profile() {
        let mut data = AppData::default();
        let created = create_profile_in_data(&mut data);
        let fallback_id = data.cat_profiles[0].id.clone();

        delete_profile_in_data(&mut data, &created.id).unwrap();

        assert_eq!(data.active_cat_profile_id, fallback_id);
        assert_eq!(data.cat_profiles.len(), 1);
    }

    #[test]
    fn deleting_last_profile_is_rejected() {
        let mut data = AppData::default();
        let only_id = data.active_cat_profile().id.clone();

        let err = delete_profile_in_data(&mut data, &only_id).unwrap_err();

        assert!(err.contains("at least one"));
    }

    #[test]
    fn set_active_profile_rejects_unknown_id() {
        let mut data = AppData::default();
        let err = set_active_profile_in_data(&mut data, "missing").unwrap_err();
        assert!(err.contains("not found"));
    }
}
