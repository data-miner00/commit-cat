use commit_cat_server::{db, AppState, build_router};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let db = db::init_db().await.expect("Failed to initialize database");
    let state = AppState { db };

    let app = build_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("CommitCat API server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
