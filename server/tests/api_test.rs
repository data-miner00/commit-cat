use axum::body::Body;
use axum::http::{Request, StatusCode};
use commit_cat_server::{AppState, build_router, db};
use http_body_util::BodyExt;
use tower::ServiceExt;

async fn setup_app() -> axum::Router {
    let pool = db::init_db_with_url("sqlite::memory:")
        .await
        .expect("in-memory DB init failed");
    let state = AppState { db: pool };
    build_router(state)
}

#[tokio::test]
async fn health_returns_200_ok() {
    let app = setup_app().await;

    let response = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"ok");
}

#[tokio::test]
async fn badge_nonexistent_user_returns_404_with_svg_content_type() {
    let app = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/badge/nonexistent_user_xyz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let content_type = response
        .headers()
        .get("content-type")
        .expect("content-type header should be present")
        .to_str()
        .unwrap();
    assert_eq!(content_type, "image/svg+xml");
}

#[tokio::test]
async fn stats_returns_200_with_total_users_and_total_commits_fields() {
    let app = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/stats")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value =
        serde_json::from_slice(&body).expect("response should be valid JSON");

    assert!(
        json.get("totalUsers").is_some(),
        "response JSON should contain 'totalUsers' field"
    );
    assert!(
        json.get("totalCommits").is_some(),
        "response JSON should contain 'totalCommits' field"
    );
}

#[tokio::test]
async fn profile_nonexistent_user_returns_json_with_error_field() {
    let app = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/profile/nonexistent_user_xyz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value =
        serde_json::from_slice(&body).expect("response should be valid JSON");

    assert!(
        json.get("error").is_some(),
        "response JSON should contain 'error' field for unknown user"
    );
}
