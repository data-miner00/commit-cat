use axum::extract::{Path, State};
use axum::Json;
use crate::AppState;

pub async fn get_profile(
    State(_state): State<AppState>,
    Path(username): Path<String>,
) -> Json<serde_json::Value> {
    // TODO: Look up user from DB and return public profile
    Json(serde_json::json!({
        "username": username,
        "level": 1,
        "streak": 0,
        "totalCommits": 0,
    }))
}
