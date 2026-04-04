use axum::extract::State;
use axum::Json;
use crate::AppState;

pub async fn get_stats(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let users: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let commits: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(total_commits), 0) FROM user_data"
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    let levels: f64 = sqlx::query_scalar(
        "SELECT COALESCE(AVG(level), 0) FROM user_data"
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(0.0);

    let active_streaks: i32 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_data WHERE current_streak > 0"
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    Json(serde_json::json!({
        "totalUsers": users,
        "totalCommits": commits,
        "averageLevel": (levels * 10.0).round() / 10.0,
        "activeStreaks": active_streaks,
    }))
}
