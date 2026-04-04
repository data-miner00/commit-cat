use axum::extract::{Query, State};
use axum::response::Redirect;
use serde::Deserialize;
use crate::AppState;

#[derive(Deserialize)]
pub struct GithubCallbackQuery {
    pub code: String,
}

pub async fn github_login() -> Redirect {
    let client_id = std::env::var("GITHUB_CLIENT_ID").unwrap_or_default();
    let redirect_uri = std::env::var("REDIRECT_URI")
        .unwrap_or_else(|_| "http://localhost:3000/auth/github/callback".to_string());
    Redirect::temporary(&format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope=read:user",
        client_id, redirect_uri
    ))
}

pub async fn github_callback(
    State(_state): State<AppState>,
    Query(query): Query<GithubCallbackQuery>,
) -> String {
    // TODO: Exchange code for access token, create/update user
    let preview_len = query.code.len().min(8);
    format!("GitHub callback received with code: {}...", &query.code[..preview_len])
}
