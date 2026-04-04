mod routes;
mod db;
mod auth;

use axum::{Router, routing::{get, put}};
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
        .route("/api/v1/profile/{username}", get(routes::profile::get_profile))
        .route("/health", get(|| async { "ok" }))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("CommitCat API server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
