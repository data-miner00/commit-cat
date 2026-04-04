use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use serde::{Deserialize, Serialize};
use crate::AppState;

#[derive(Deserialize)]
pub struct SyncRequest {
    pub device_id: String,
    pub events: Vec<SyncEvent>,
    pub snapshot: UserSnapshot,
}

#[derive(Deserialize)]
pub struct SyncEvent {
    pub event_type: String,
    pub xp: u32,
    pub occurred_at: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct UserSnapshot {
    pub level: u32,
    pub exp: u32,
    pub total_commits: u32,
    pub current_streak: u32,
    pub current_hat: Option<String>,
    pub unlocked_hats: Vec<String>,
}

#[derive(Serialize)]
pub struct SyncResponse {
    pub ok: bool,
    pub merged: UserSnapshot,
}

fn extract_user_id(headers: &HeaderMap, db: &sqlx::SqlitePool) -> Option<String> {
    let auth = headers.get("Authorization")?.to_str().ok()?;
    let token = auth.strip_prefix("Bearer ")?;
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "commitcat-dev-secret".to_string());
    let claims = crate::auth::verify_token(token, &secret).ok()?;
    let _ = db; // will be used for token validation later
    Some(claims.sub)
}

pub async fn sync_data(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<SyncRequest>,
) -> Json<serde_json::Value> {
    let user_id = match extract_user_id(&headers, &state.db) {
        Some(id) => id,
        None => return Json(serde_json::json!({ "ok": false, "error": "Unauthorized" })),
    };

    // Store sync events
    for event in &req.events {
        let _ = sqlx::query(
            "INSERT INTO sync_events (user_id, device_id, event_type, xp, occurred_at)
             VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&user_id)
        .bind(&req.device_id)
        .bind(&event.event_type)
        .bind(event.xp)
        .bind(&event.occurred_at)
        .execute(&state.db)
        .await;
    }

    // Update user_data with latest snapshot
    let unlocked_json = serde_json::to_string(&req.snapshot.unlocked_hats).unwrap_or_default();
    let _ = sqlx::query(
        "UPDATE user_data SET
            level = MAX(level, ?),
            exp = ?,
            total_commits = MAX(total_commits, ?),
            current_streak = MAX(current_streak, ?),
            longest_streak = MAX(longest_streak, ?),
            current_hat = ?,
            unlocked_hats = ?,
            updated_at = datetime('now')
         WHERE user_id = ?"
    )
    .bind(req.snapshot.level)
    .bind(req.snapshot.exp)
    .bind(req.snapshot.total_commits)
    .bind(req.snapshot.current_streak)
    .bind(req.snapshot.current_streak)
    .bind(&req.snapshot.current_hat)
    .bind(&unlocked_json)
    .bind(&user_id)
    .execute(&state.db)
    .await;

    Json(serde_json::json!({
        "ok": true,
        "merged": req.snapshot,
    }))
}

pub async fn get_data(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<serde_json::Value> {
    let user_id = match extract_user_id(&headers, &state.db) {
        Some(id) => id,
        None => return Json(serde_json::json!({ "ok": false, "error": "Unauthorized" })),
    };

    let row = sqlx::query_as::<_, (i32, i32, i32, i32, i32, Option<String>, String)>(
        "SELECT level, exp, total_commits, current_streak, longest_streak, current_hat, unlocked_hats
         FROM user_data WHERE user_id = ?"
    )
    .bind(&user_id)
    .fetch_optional(&state.db)
    .await;

    match row {
        Ok(Some((level, exp, commits, streak, longest, hat, hats_json))) => {
            let unlocked: Vec<String> = serde_json::from_str(&hats_json).unwrap_or_default();
            Json(serde_json::json!({
                "ok": true,
                "data": {
                    "level": level,
                    "exp": exp,
                    "totalCommits": commits,
                    "currentStreak": streak,
                    "longestStreak": longest,
                    "currentHat": hat,
                    "unlockedHats": unlocked,
                }
            }))
        }
        _ => Json(serde_json::json!({ "ok": false, "error": "User data not found" })),
    }
}
