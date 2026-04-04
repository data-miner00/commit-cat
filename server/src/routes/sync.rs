use axum::extract::State;
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

#[derive(Deserialize, Serialize)]
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

pub async fn sync_data(
    State(_state): State<AppState>,
    Json(req): Json<SyncRequest>,
) -> Json<SyncResponse> {
    // TODO: Authenticate user from Authorization header
    // TODO: Store events, merge snapshots, return merged state
    Json(SyncResponse {
        ok: true,
        merged: req.snapshot,
    })
}

pub async fn get_data(
    State(_state): State<AppState>,
) -> Json<serde_json::Value> {
    // TODO: Authenticate and return user's current data
    Json(serde_json::json!({ "ok": true, "message": "not implemented" }))
}
