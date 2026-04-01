use crate::services::storage;
use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use chrono::Local;
use commit_cat_core::models::activity::ActivityEvent;
use commit_cat_core::models::growth::exp_for_level;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::net::TcpListener;

const PORT: u16 = 39547;

#[derive(Deserialize)]
#[allow(dead_code)]
struct ActivityRequest {
    r#type: String,
    #[serde(default)]
    seconds: Option<u64>,
    #[serde(default)]
    filename: Option<String>,
    #[serde(default)]
    language: Option<String>,
}

#[derive(Serialize)]
struct ActivityResponse {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

/// XP 추가 (github.rs/docker.rs와 동일 패턴)
fn add_xp_internal(app: &AppHandle, amount: u32, source: &str) -> Result<(), String> {
    let mut data = storage::load(app)?;

    data.cat.exp += amount;
    let mut leveled_up = false;

    let now = Local::now().format("%H:%M").to_string();
    data.today.events.push(ActivityEvent {
        timestamp: now,
        event_type: source.to_string(),
        xp: amount,
        detail: source.to_string(),
    });

    loop {
        let needed = exp_for_level(data.cat.level);
        if data.cat.exp >= needed {
            data.cat.exp -= needed;
            data.cat.level += 1;
            leveled_up = true;
        } else {
            break;
        }
    }

    if leveled_up {
        let now = Local::now().format("%H:%M").to_string();
        data.today.events.push(ActivityEvent {
            timestamp: now,
            event_type: "level_up".to_string(),
            xp: 0,
            detail: format!("Lv.{}", data.cat.level),
        });
    }

    data.today.exp_gained += amount;
    storage::save(app, &data)?;

    if leveled_up {
        let _ = app.emit("xp:level-up", data.cat.level);
    }

    Ok(())
}

async fn handle_activity(
    State(app): State<AppHandle>,
    Json(req): Json<ActivityRequest>,
) -> (StatusCode, Json<ActivityResponse>) {
    let result = match req.r#type.as_str() {
        "coding_time" => {
            let _ = app.emit("plugin:coding-active", ());
            Ok("coding time recorded")
        }
        "file_change" => {
            let detail = req.filename.unwrap_or_default();
            let _ = app.emit("plugin:file-change", &detail);
            Ok("file change recorded")
        }
        "save" => {
            let _ = app.emit("plugin:save", ());
            Ok("save recorded")
        }
        "build_success" => {
            let _ = app.emit("plugin:build-success", ());
            add_xp_internal(&app, 15, "build_success").map(|_| "build success +15 XP")
        }
        "build_fail" => {
            let _ = app.emit("plugin:build-fail", ());
            Ok("build fail recorded")
        }
        _ => Err(format!("unknown type: {}", req.r#type)),
    };

    match result {
        Ok(msg) => (
            StatusCode::OK,
            Json(ActivityResponse {
                ok: true,
                message: Some(msg.to_string()),
            }),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ActivityResponse {
                ok: false,
                message: Some(e),
            }),
        ),
    }
}

/// 플러그인 HTTP 서버 시작 (localhost:39547)
pub async fn start_server(app: AppHandle) {
    let router = Router::new()
        .route("/activity", post(handle_activity))
        .with_state(app);

    let addr = format!("127.0.0.1:{}", PORT);
    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[plugin_server] Failed to bind {}: {}", addr, e);
            return;
        }
    };

    eprintln!("[plugin_server] Listening on {}", addr);
    if let Err(e) = axum::serve(listener, router).await {
        eprintln!("[plugin_server] Server error: {}", e);
    }
}
