use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect};
use axum::Json;
use serde::{Deserialize, Serialize};
use crate::AppState;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct GithubCallbackQuery {
    pub code: String,
}

#[derive(Deserialize)]
struct GithubTokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct GithubUser {
    id: i64,
    login: String,
    avatar_url: Option<String>,
}

#[derive(Serialize)]
struct AuthResponse {
    token: String,
    username: String,
    avatar_url: Option<String>,
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
    State(state): State<AppState>,
    Query(query): Query<GithubCallbackQuery>,
) -> impl IntoResponse {
    match exchange_and_register(state, &query.code).await {
        Ok(response) => Json(response).into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Auth failed: {}", e),
        ).into_response(),
    }
}

async fn exchange_and_register(state: AppState, code: &str) -> Result<AuthResponse, String> {
    let client_id = std::env::var("GITHUB_CLIENT_ID").map_err(|_| "GITHUB_CLIENT_ID not set")?;
    let client_secret = std::env::var("GITHUB_CLIENT_SECRET").map_err(|_| "GITHUB_CLIENT_SECRET not set")?;

    // 1. Exchange code for access token
    let client = reqwest::Client::new();
    let token_res: GithubTokenResponse = client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .json(&serde_json::json!({
            "client_id": client_id,
            "client_secret": client_secret,
            "code": code,
        }))
        .send()
        .await
        .map_err(|e| format!("Token request failed: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Token parse failed: {}", e))?;

    // 2. Get GitHub user info
    let github_user: GithubUser = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", token_res.access_token))
        .header("User-Agent", "CommitCat")
        .send()
        .await
        .map_err(|e| format!("User request failed: {}", e))?
        .json()
        .await
        .map_err(|e| format!("User parse failed: {}", e))?;

    // 3. Upsert user in DB
    let user_id = format!("user-{}", Uuid::new_v4());
    sqlx::query(
        "INSERT INTO users (id, github_id, github_username, github_avatar_url, access_token)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT(github_id) DO UPDATE SET
            github_username = excluded.github_username,
            github_avatar_url = excluded.github_avatar_url,
            access_token = excluded.access_token,
            updated_at = datetime('now')"
    )
    .bind(&user_id)
    .bind(github_user.id)
    .bind(&github_user.login)
    .bind(&github_user.avatar_url)
    .bind(&token_res.access_token)
    .execute(&state.db)
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    // Get actual user_id (in case of conflict, the existing id is kept)
    let actual_id: String = sqlx::query_scalar("SELECT id FROM users WHERE github_id = ?")
        .bind(github_user.id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| format!("DB fetch error: {}", e))?;

    // 4. Ensure user_data row exists
    sqlx::query(
        "INSERT OR IGNORE INTO user_data (user_id) VALUES (?)"
    )
    .bind(&actual_id)
    .execute(&state.db)
    .await
    .map_err(|e| format!("DB error: {}", e))?;

    // 5. Generate JWT
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "commitcat-dev-secret".to_string());
    let token = crate::auth::create_token(&actual_id, &github_user.login, &jwt_secret)
        .map_err(|e| format!("JWT error: {}", e))?;

    Ok(AuthResponse {
        token,
        username: github_user.login,
        avatar_url: github_user.avatar_url,
    })
}
