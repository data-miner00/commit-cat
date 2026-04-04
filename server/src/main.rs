mod routes;
mod db;
mod auth;

use axum::{Router, routing::{get, put}, response::Html};
use tower_http::cors::CorsLayer;
use std::net::SocketAddr;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let db = db::init_db().await.expect("Failed to initialize database");
    let state = AppState { db };

    let app = Router::new()
        .route("/auth/github", get(routes::auth::github_login))
        .route("/auth/github/callback", get(routes::auth::github_callback))
        .route("/api/v1/sync", put(routes::sync::sync_data))
        .route("/api/v1/sync", get(routes::sync::get_data))
        .route("/badge/{username}", get(routes::badge::get_badge))
        .route("/profile/{username}", get(routes::profile::get_profile_page))
        .route("/api/v1/profile/{username}", get(routes::profile::get_profile))
        .route("/api/v1/stats", get(routes::stats::get_stats))
        .route("/health", get(|| async { "ok" }))
        .route("/", get(landing))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("CommitCat API server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn landing() -> Html<&'static str> {
    Html(r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>CommitCat API</title>
<style>
  * { margin: 0; padding: 0; box-sizing: border-box; }
  body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    background: #0d1117;
    color: #e6edf3;
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .container { text-align: center; max-width: 480px; padding: 20px; }
  h1 { font-size: 48px; margin-bottom: 8px; }
  .tagline { color: #8b949e; font-size: 16px; margin-bottom: 32px; }
  .endpoints { text-align: left; background: #161b22; border: 1px solid #30363d; border-radius: 12px; padding: 20px; }
  .endpoints h2 { font-size: 14px; color: #8b949e; text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 12px; }
  .ep { margin-bottom: 10px; }
  .method { display: inline-block; width: 36px; font-size: 11px; font-weight: 700; color: #3fb950; }
  .method.put { color: #d29922; }
  .path { color: #58a6ff; font-family: monospace; font-size: 14px; }
  .desc { color: #8b949e; font-size: 12px; margin-left: 40px; }
  .footer { margin-top: 24px; color: #484f58; font-size: 12px; }
  .footer a { color: #58a6ff; text-decoration: none; }
</style>
</head>
<body>
<div class="container">
  <h1>🐱</h1>
  <p class="tagline">CommitCat Cloud API</p>
  <div class="endpoints">
    <h2>Endpoints</h2>
    <div class="ep">
      <span class="method">GET</span> <span class="path">/badge/{username}</span>
      <div class="desc">Animated SVG badge for GitHub README</div>
    </div>
    <div class="ep">
      <span class="method">GET</span> <span class="path">/profile/{username}</span>
      <div class="desc">Public profile page</div>
    </div>
    <div class="ep">
      <span class="method">GET</span> <span class="path">/api/v1/profile/{username}</span>
      <div class="desc">Profile data (JSON)</div>
    </div>
    <div class="ep">
      <span class="method">GET</span> <span class="path">/api/v1/stats</span>
      <div class="desc">Global stats (users, commits, levels)</div>
    </div>
    <div class="ep">
      <span class="method">GET</span> <span class="path">/auth/github</span>
      <div class="desc">GitHub OAuth login</div>
    </div>
    <div class="ep">
      <span class="method put">PUT</span> <span class="path">/api/v1/sync</span>
      <div class="desc">Sync device data (auth required)</div>
    </div>
    <div class="ep">
      <span class="method">GET</span> <span class="path">/api/v1/sync</span>
      <div class="desc">Get synced data (auth required)</div>
    </div>
  </div>
  <div class="footer">
    <a href="https://github.com/eunseo9311/commit-cat">GitHub</a> · MIT
  </div>
</div>
</body>
</html>"#)
}
