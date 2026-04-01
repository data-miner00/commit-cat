use crate::services::storage;
use chrono::Local;
use reqwest::header::{ACCEPT, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

const REPO: &str = "eunseo9311/commit-cat";

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateAvailable {
    latest_version: String,
}

/// Parse "X.Y.Z" into (X, Y, Z) tuple, stripping optional "v" prefix
fn parse_version(v: &str) -> Option<(u32, u32, u32)> {
    let v = v.strip_prefix('v').unwrap_or(v);
    let parts: Vec<&str> = v.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    Some((
        parts[0].parse().ok()?,
        parts[1].parse().ok()?,
        parts[2].parse().ok()?,
    ))
}

/// Check for updates against GitHub releases. Returns Some(version) if newer.
pub async fn check_update(app: &AppHandle) -> Result<Option<String>, String> {
    let client = reqwest::Client::new();
    let url = format!("https://api.github.com/repos/{}/releases/latest", REPO);

    let res = client
        .get(&url)
        .header(USER_AGENT, "commit-cat")
        .header(ACCEPT, "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| format!("Update check failed: {}", e))?;

    if !res.status().is_success() {
        return Err(format!("GitHub API status: {}", res.status()));
    }

    let release: GitHubRelease = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse release: {}", e))?;

    let current = env!("CARGO_PKG_VERSION");
    let latest_str = release
        .tag_name
        .strip_prefix('v')
        .unwrap_or(&release.tag_name);

    let current_ver = parse_version(current);
    let latest_ver = parse_version(latest_str);

    // Update last_update_check date
    let today = Local::now().format("%Y-%m-%d").to_string();
    if let Ok(mut data) = storage::load(app) {
        data.last_update_check = Some(today);
        let _ = storage::save(app, &data);
    }

    if let (Some(cur), Some(lat)) = (current_ver, latest_ver) {
        if lat > cur {
            let _ = app.emit(
                "update:available",
                UpdateAvailable {
                    latest_version: latest_str.to_string(),
                },
            );
            return Ok(Some(latest_str.to_string()));
        }
    }

    Ok(None)
}

/// Background update checker — runs on startup then every 24 hours
pub async fn start_update_checker(app: AppHandle) {
    // Initial delay to let the app settle
    tokio::time::sleep(Duration::from_secs(10)).await;

    loop {
        // Check if already checked today
        let should_check = match storage::load(&app) {
            Ok(data) => {
                let today = Local::now().format("%Y-%m-%d").to_string();
                data.last_update_check.as_deref() != Some(&today)
            }
            Err(_) => true,
        };

        if should_check {
            let _ = check_update(&app).await;
        }

        // Sleep 24 hours before next check
        tokio::time::sleep(Duration::from_secs(86400)).await;
    }
}
