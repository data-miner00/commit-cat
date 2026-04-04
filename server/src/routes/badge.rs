use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::http::{header, StatusCode, HeaderMap};
use crate::AppState;

fn svg_etag(svg: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    svg.hash(&mut hasher);
    format!("\"{}\"", hasher.finish())
}

pub async fn get_badge(
    State(state): State<AppState>,
    Path(username): Path<String>,
    headers: HeaderMap,
) -> Response {
    // Look up user data from DB
    let row = sqlx::query_as::<_, (i32, i32, i32)>(
        "SELECT ud.level, ud.current_streak, ud.total_commits
         FROM users u
         JOIN user_data ud ON u.id = ud.user_id
         WHERE u.github_username = ?"
    )
    .bind(&username)
    .fetch_optional(&state.db)
    .await;

    let (level, streak, commits) = match row {
        Ok(Some(data)) => (data.0 as u32, data.1 as u32, data.2 as u32),
        _ => {
            let svg = commit_cat_core::badge::generate_badge(0, 0, 0, &username);
            return (
                StatusCode::NOT_FOUND,
                [(header::CONTENT_TYPE, "image/svg+xml")],
                svg,
            ).into_response();
        }
    };

    let svg = commit_cat_core::badge::generate_badge(level, streak, commits, &username);
    let etag = svg_etag(&svg);

    // ETag match → 304 Not Modified
    if let Some(if_none_match) = headers.get(header::IF_NONE_MATCH) {
        if let Ok(val) = if_none_match.to_str() {
            if val == etag {
                return StatusCode::NOT_MODIFIED.into_response();
            }
        }
    }

    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "image/svg+xml"),
            (header::CACHE_CONTROL, "public, max-age=300"),
            (header::ETAG, &etag),
        ],
        svg,
    ).into_response()
}
