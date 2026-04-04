use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::http::{header, StatusCode};
use crate::AppState;

pub async fn get_badge(
    State(_state): State<AppState>,
    Path(username): Path<String>,
) -> Response {
    // TODO: Look up user data from DB
    let level = 1;
    let streak = 0;
    let commits = 0;

    let svg = commit_cat_core::badge::generate_badge(level, streak, commits, &username);

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "image/svg+xml"),
         (header::CACHE_CONTROL, "no-cache, no-store, must-revalidate")],
        svg,
    ).into_response()
}
