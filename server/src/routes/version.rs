use axum::Json;

pub async fn get_version() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "name": env!("CARGO_PKG_NAME"),
        "version": env!("CARGO_PKG_VERSION"),
        "commit": option_env!("GIT_SHA").unwrap_or("unknown"),
    }))
}
