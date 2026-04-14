use axum::body::Body;
use axum::http::{Request, StatusCode};
use commit_cat_server::{AppState, build_router, db};
use http_body_util::BodyExt;
use sqlx::SqlitePool;
use tower::ServiceExt;

async fn setup_app() -> axum::Router {
    let pool = db::init_db_with_url("sqlite::memory:")
        .await
        .expect("in-memory DB init failed");
    let state = AppState { db: pool };
    build_router(state)
}

async fn setup_app_with_data() -> (axum::Router, SqlitePool) {
    let pool = db::init_db_with_url("sqlite::memory:")
        .await
        .expect("in-memory DB init failed");
    seed_users(&pool).await;
    let state = AppState { db: pool.clone() };
    (build_router(state), pool)
}

async fn seed_users(pool: &SqlitePool) {
    let users = [
        ("u1", 1i64, "alice", 10i32, 500i32, 2i32),
        ("u2", 2i64, "bob", 20i32, 1200i32, 7i32),
        ("u3", 3i64, "carol", 5i32, 80i32, 2i32),
    ];
    for (id, gh_id, name, level, commits, streak) in users {
        sqlx::query(
            "INSERT INTO users (id, github_id, github_username, access_token)
             VALUES (?, ?, ?, 'tok')",
        )
        .bind(id)
        .bind(gh_id)
        .bind(name)
        .execute(pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO user_data (user_id, level, exp, total_commits, current_streak, longest_streak)
             VALUES (?, ?, 0, ?, ?, 14)",
        )
        .bind(id)
        .bind(level)
        .bind(commits)
        .bind(streak)
        .execute(pool)
        .await
        .unwrap();
    }
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
async fn badge_returns_svg_content_type() {
    // Badge is generated from GitHub contributions scraping now (no DB lookup),
    // so it always returns a valid SVG regardless of DB state.
    let app = setup_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/badge/octocat")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

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
async fn stats_reflects_seeded_data() {
    let (app, _pool) = setup_app_with_data().await;

    let response = app
        .oneshot(Request::builder().uri("/api/v1/stats").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["totalUsers"], 3);
    assert_eq!(json["totalCommits"], 500 + 1200 + 80);
    assert_eq!(json["activeStreaks"], 3);
}

#[tokio::test]
async fn leaderboard_json_sorts_by_level_by_default() {
    let (app, _pool) = setup_app_with_data().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/leaderboard")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let entries = json["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0]["username"], "bob");
    assert_eq!(entries[0]["level"], 20);
    assert_eq!(entries[1]["username"], "alice");
    assert_eq!(entries[2]["username"], "carol");
}

#[tokio::test]
async fn leaderboard_json_sorts_by_commits() {
    let (app, _pool) = setup_app_with_data().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/leaderboard?sort=commits")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let entries = json["entries"].as_array().unwrap();

    assert_eq!(entries[0]["username"], "bob");
    assert_eq!(entries[0]["totalCommits"], 1200);
}

#[tokio::test]
async fn leaderboard_page_returns_html() {
    let (app, _pool) = setup_app_with_data().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/leaderboard")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let html = std::str::from_utf8(&body).unwrap();
    assert!(html.contains("Leaderboard"));
    assert!(html.contains("bob"));
    assert!(html.contains("alice"));
}

#[tokio::test]
async fn profile_activity_returns_recent_events() {
    let (app, pool) = setup_app_with_data().await;

    // Seed a few sync events for alice
    for (et, xp) in [("commit", 10), ("push", 5), ("commit", 10)] {
        sqlx::query(
            "INSERT INTO sync_events (user_id, device_id, event_type, xp, occurred_at)
             VALUES ('u1', 'dev1', ?, ?, datetime('now'))"
        )
        .bind(et)
        .bind(xp as i32)
        .execute(&pool)
        .await
        .unwrap();
    }

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/profile/alice/activity")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["username"], "alice");
    let events = json["events"].as_array().unwrap();
    assert_eq!(events.len(), 3);
    assert_eq!(events[0]["xp"], 10);
}

#[tokio::test]
async fn profile_page_includes_rank() {
    let (app, _pool) = setup_app_with_data().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/profile/bob")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let html = std::str::from_utf8(&body).unwrap();
    // bob has the highest level so rank should be #1
    assert!(html.contains("Rank #1"), "expected 'Rank #1' in profile page");
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
