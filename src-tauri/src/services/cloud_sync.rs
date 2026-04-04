use crate::services::storage;
use chrono::Local;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

#[derive(Serialize)]
struct SyncRequest {
    device_id: String,
    events: Vec<SyncEvent>,
    snapshot: UserSnapshot,
}

#[derive(Serialize)]
struct SyncEvent {
    event_type: String,
    xp: u32,
    occurred_at: String,
}

#[derive(Serialize, Deserialize)]
struct UserSnapshot {
    level: u32,
    exp: u32,
    total_commits: u32,
    current_streak: u32,
    current_hat: Option<String>,
    unlocked_hats: Vec<String>,
}

#[derive(Deserialize)]
struct SyncResponse {
    ok: bool,
    #[allow(dead_code)]
    merged: Option<UserSnapshot>,
}

/// 5분마다 클라우드에 데이터 싱크
pub async fn start_sync_service(app: AppHandle) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));

    loop {
        interval.tick().await;

        let data = match storage::load(&app) {
            Ok(d) => d,
            Err(_) => continue,
        };

        // 클라우드 싱크 비활성이면 스킵
        let cloud_token = match &data.settings.cloud_token {
            Some(t) if !t.is_empty() => t.clone(),
            _ => continue,
        };

        let server_url = data.settings.cloud_server_url.clone()
            .unwrap_or_else(|| "https://api.commitcat.dev".to_string());

        let device_id = data.settings.device_id.clone()
            .unwrap_or_else(|| {
                let id = format!("device-{}", uuid::Uuid::new_v4());
                // Save device_id for future syncs
                if let Ok(mut d) = storage::load(&app) {
                    d.settings.device_id = Some(id.clone());
                    let _ = storage::save(&app, &d);
                }
                id
            });

        let snapshot = UserSnapshot {
            level: data.cat.level,
            exp: data.cat.exp,
            total_commits: data.cat.total_commits,
            current_streak: data.cat.streak_days,
            current_hat: data.cat.current_hat.clone(),
            unlocked_hats: data.cat.unlocked_hats.clone(),
        };

        let req = SyncRequest {
            device_id,
            events: vec![], // TODO: drain pending events queue
            snapshot,
        };

        let client = reqwest::Client::new();
        match client
            .put(format!("{}/api/v1/sync", server_url))
            .header("Authorization", format!("Bearer {}", cloud_token))
            .json(&req)
            .send()
            .await
        {
            Ok(res) => {
                if let Ok(sync_res) = res.json::<SyncResponse>().await {
                    if sync_res.ok {
                        let _ = app.emit("cloud:synced", Local::now().format("%H:%M").to_string());
                    }
                }
            }
            Err(_) => {
                // 네트워크 실패 — 다음 틱에서 재시도 (오프라인 내성)
                continue;
            }
        }
    }
}
