use axum::extract::{Path, State};
use axum::Json;
use crate::AppState;

pub async fn get_profile(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Json<serde_json::Value> {
    let row = sqlx::query_as::<_, (i32, i32, i32, i32, i32, Option<String>, String, Option<String>)>(
        "SELECT ud.level, ud.exp, ud.total_commits, ud.current_streak, ud.longest_streak,
                ud.current_hat, ud.unlocked_hats, u.github_avatar_url
         FROM users u
         JOIN user_data ud ON u.id = ud.user_id
         WHERE u.github_username = ?"
    )
    .bind(&username)
    .fetch_optional(&state.db)
    .await;

    match row {
        Ok(Some((level, exp, commits, streak, longest, hat, hats_json, avatar))) => {
            let unlocked: Vec<String> = serde_json::from_str(&hats_json).unwrap_or_default();
            Json(serde_json::json!({
                "username": username,
                "avatarUrl": avatar,
                "level": level,
                "exp": exp,
                "totalCommits": commits,
                "currentStreak": streak,
                "longestStreak": longest,
                "currentHat": hat,
                "unlockedHats": unlocked,
            }))
        }
        _ => Json(serde_json::json!({
            "error": "User not found",
            "username": username,
        })),
    }
}
