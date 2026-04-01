use crate::services::storage;
use chrono::{Datelike, Local};
use commit_cat_core::models::activity::ActivityEvent;
use commit_cat_core::models::growth::{exp_for_level, LevelInfo};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddXpResult {
    pub level: u32,
    pub current_exp: u32,
    pub exp_to_next: u32,
    pub leveled_up: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreakInfo {
    pub streak_days: u32,
    pub last_active_date: Option<String>,
}

#[tauri::command]
pub async fn get_xp_status(app: AppHandle) -> Result<LevelInfo, String> {
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
pub async fn get_streak_info(app: AppHandle) -> Result<StreakInfo, String> {
    let data = storage::load(&app)?;
    Ok(StreakInfo {
        streak_days: data.cat.streak_days,
        last_active_date: data.cat.last_active_date.clone(),
    })
}

#[tauri::command]
pub async fn add_xp(app: AppHandle, amount: u32, source: String) -> Result<AddXpResult, String> {
    let mut data = storage::load(&app)?;

    // 날짜 보정: today.date가 오늘이 아니면 리셋
    let today_date = Local::now().format("%Y-%m-%d").to_string();
    if data.today.date != today_date {
        data.today = commit_cat_core::models::activity::DailySummary {
            date: today_date,
            ..Default::default()
        };
    }

    data.cat.exp += amount;
    let mut leveled_up = false;

    // 이벤트 기록
    let now = Local::now().format("%H:%M").to_string();
    data.today.events.push(ActivityEvent {
        timestamp: now.clone(),
        event_type: source.clone(),
        xp: amount,
        detail: source.clone(),
    });

    // 레벨업 루프
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

    // 레벨업 이벤트 기록
    if leveled_up {
        let now = Local::now().format("%H:%M").to_string();
        data.today.events.push(ActivityEvent {
            timestamp: now,
            event_type: "level_up".to_string(),
            xp: 0,
            detail: format!("Lv.{}", data.cat.level),
        });
    }

    // source별 통계 업데이트
    data.today.exp_gained += amount;
    match source.as_str() {
        "commit" => {
            data.cat.total_commits += 1;
            data.today.commits += 1;
        }
        "coding_hour" => {
            data.cat.total_coding_minutes += 60;
            data.today.coding_minutes += 60;
        }
        _ => {}
    }

    // ── Streak 업데이트 ──
    let today_str = Local::now().format("%Y-%m-%d").to_string();
    let mut streak_milestone: Option<(u32, u32)> = None; // (days, bonus_xp)

    if data.cat.last_active_date.as_deref() != Some(&today_str) {
        let yesterday = (Local::now() - chrono::Duration::days(1))
            .format("%Y-%m-%d")
            .to_string();

        if data.cat.last_active_date.as_deref() == Some(&yesterday) {
            data.cat.streak_days += 1;
        } else {
            data.cat.streak_days = 1;
        }
        data.cat.last_active_date = Some(today_str);

        // 마일스톤 체크
        let bonus = match data.cat.streak_days {
            3 => Some(50u32),
            7 => Some(100),
            30 => Some(500),
            _ => None,
        };
        if let Some(bonus_xp) = bonus {
            data.cat.exp += bonus_xp;
            data.today.exp_gained += bonus_xp;

            // 보너스 XP 이벤트 기록
            let ts = Local::now().format("%H:%M").to_string();
            data.today.events.push(ActivityEvent {
                timestamp: ts,
                event_type: "streak".to_string(),
                xp: bonus_xp,
                detail: format!("{} day streak!", data.cat.streak_days),
            });

            // 보너스로 인한 추가 레벨업 체크
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

            streak_milestone = Some((data.cat.streak_days, bonus_xp));
        }
    }

    // Check hat unlocks
    let month = Local::now().month();
    let newly_unlocked = commit_cat_core::items::check_unlocks(
        data.cat.level,
        data.cat.total_commits,
        data.cat.streak_days,
        data.cat.total_late_night_sessions,
        month,
        &data.cat.unlocked_hats,
    );
    for hat in &newly_unlocked {
        if !data.cat.unlocked_hats.contains(hat) {
            data.cat.unlocked_hats.push(hat.clone());
        }
    }
    if !newly_unlocked.is_empty() {
        // 자동 장착: 현재 장착 중인 아이템이 없으면 첫 해금 아이템을 장착
        if let Some(auto_hat) =
            commit_cat_core::items::auto_equip(&newly_unlocked, &data.cat.current_hat)
        {
            data.cat.current_hat = Some(auto_hat.clone());
            let _ = app.emit("hat:equipped", &auto_hat);
        }
        let _ = app.emit("hat:unlocked", &newly_unlocked);
    }

    // 이벤트 기반 자동 장착 (생일, 레벨, 스트릭, 12월)
    let is_birthday = {
        let today = Local::now();
        data.settings.birthday_month == Some(today.month())
            && data.settings.birthday_day == Some(today.day())
    };
    if let Some(event) = commit_cat_core::items::check_event_auto_equip(
        is_birthday,
        data.cat.level,
        data.cat.streak_days,
        month,
        &data.cat.unlocked_hats,
    ) {
        // 기존 자동 장착이 만료되었거나 없을 때만 적용
        let expired = data.cat.auto_equip_until.as_ref().map_or(true, |until| {
            chrono::NaiveDateTime::parse_from_str(until, "%Y-%m-%dT%H:%M:%S")
                .map_or(true, |t| Local::now().naive_local() > t)
        });
        if expired && data.cat.current_hat.as_deref() != Some(&event.hat_id) {
            if data.cat.preferred_hat.is_none() {
                data.cat.preferred_hat = data.cat.current_hat.clone();
            }
            data.cat.current_hat = Some(event.hat_id.clone());
            let until = (Local::now() + chrono::Duration::hours(event.duration_hours as i64))
                .naive_local()
                .format("%Y-%m-%dT%H:%M:%S")
                .to_string();
            data.cat.auto_equip_until = Some(until);
            let _ = app.emit("hat:equipped", &event.hat_id);
            let _ = app.emit("hat:event-equip", &event.reason);
        }
    }

    let result = AddXpResult {
        level: data.cat.level,
        current_exp: data.cat.exp,
        exp_to_next: exp_for_level(data.cat.level),
        leveled_up,
    };

    storage::save(&app, &data)?;

    // 트레이 툴팁 streak 업데이트
    if let Some(tray) = app.tray_by_id("main-tray") {
        if data.cat.streak_days > 0 {
            let _ = tray.set_tooltip(Some(&format!(
                "CommitCat — {} day streak",
                data.cat.streak_days
            )));
        }
    }

    if leveled_up {
        let _ = app.emit("xp:level-up", result.level);
    }

    // Streak 마일스톤 이벤트 발행
    if let Some((days, bonus)) = streak_milestone {
        #[derive(Clone, Serialize)]
        struct StreakMilestone {
            days: u32,
            bonus: u32,
        }
        let _ = app.emit("streak:milestone", StreakMilestone { days, bonus });
    }

    Ok(result)
}

#[tauri::command]
pub async fn equip_hat(app: AppHandle, hat_id: Option<String>) -> Result<(), String> {
    let mut data = storage::load(&app)?;
    data.cat.current_hat = hat_id.clone();
    data.cat.preferred_hat = hat_id;
    data.cat.auto_equip_until = None;
    storage::save(&app, &data)?;
    Ok(())
}

#[tauri::command]
pub async fn unlock_all_hats(app: AppHandle) -> Result<(), String> {
    let mut data = storage::load(&app)?;
    let all_hats: Vec<String> = commit_cat_core::items::HATS
        .iter()
        .map(|h| h.id.to_string())
        .collect();
    for hat in &all_hats {
        if !data.cat.unlocked_hats.contains(hat) {
            data.cat.unlocked_hats.push(hat.clone());
        }
    }
    storage::save(&app, &data)?;
    let _ = app.emit("hat:unlocked", &all_hats);
    Ok(())
}

#[tauri::command]
pub async fn check_event_equip(app: AppHandle) -> Result<Option<String>, String> {
    let mut data = storage::load(&app)?;
    let today = Local::now();

    // 자동 장착 만료 체크 → 복원
    if let Some(until) = &data.cat.auto_equip_until {
        if let Ok(t) = chrono::NaiveDateTime::parse_from_str(until, "%Y-%m-%dT%H:%M:%S") {
            if today.naive_local() > t {
                data.cat.current_hat = data.cat.preferred_hat.take();
                data.cat.auto_equip_until = None;
                storage::save(&app, &data)?;
                let _ = app.emit("hat:equipped", &data.cat.current_hat);
            }
        }
    }

    // 이벤트 자동 장착 체크
    let is_birthday = data.settings.birthday_month == Some(today.month())
        && data.settings.birthday_day == Some(today.day());
    let month = today.month();

    if let Some(event) = commit_cat_core::items::check_event_auto_equip(
        is_birthday,
        data.cat.level,
        data.cat.streak_days,
        month,
        &data.cat.unlocked_hats,
    ) {
        let expired = data.cat.auto_equip_until.as_ref().map_or(true, |until| {
            chrono::NaiveDateTime::parse_from_str(until, "%Y-%m-%dT%H:%M:%S")
                .map_or(true, |t| today.naive_local() > t)
        });
        if expired && data.cat.current_hat.as_deref() != Some(&event.hat_id) {
            if data.cat.preferred_hat.is_none() {
                data.cat.preferred_hat = data.cat.current_hat.clone();
            }
            data.cat.current_hat = Some(event.hat_id.clone());
            let until = (today + chrono::Duration::hours(event.duration_hours as i64))
                .naive_local()
                .format("%Y-%m-%dT%H:%M:%S")
                .to_string();
            data.cat.auto_equip_until = Some(until);
            storage::save(&app, &data)?;
            let _ = app.emit("hat:equipped", &event.hat_id);
            return Ok(Some(event.reason));
        }
    }
    Ok(None)
}

#[tauri::command]
pub async fn get_hat_info(app: AppHandle) -> Result<serde_json::Value, String> {
    let data = storage::load(&app)?;
    Ok(serde_json::json!({
        "currentHat": data.cat.current_hat,
        "unlockedHats": data.cat.unlocked_hats,
    }))
}
