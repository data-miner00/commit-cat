use crate::services::storage;
use chrono::Local;
use commit_cat_core::models::activity::ActivityEvent;
use commit_cat_core::models::growth::exp_for_level;
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::Deserialize;
use std::path::Path;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

#[derive(Deserialize)]
struct GitHubUser {
    login: String,
}

#[derive(Deserialize)]
struct GitHubPR {
    number: u64,
    state: String,
    merged_at: Option<String>,
}

#[derive(Deserialize)]
struct GitHubRepo {
    stargazers_count: u32,
}

/// GitHub 토큰 검증 → username 반환
pub async fn verify_token(token: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let res = client
        .get("https://api.github.com/user")
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .header(USER_AGENT, "commit-cat")
        .header(ACCEPT, "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !res.status().is_success() {
        return Err(format!("Invalid token (status {})", res.status()));
    }

    let user: GitHubUser = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(user.login)
}

/// .git/config에서 origin remote URL → "owner/repo" 추출
fn extract_github_remote(repo_path: &Path) -> Option<String> {
    let config_path = repo_path.join(".git/config");
    let content = std::fs::read_to_string(config_path).ok()?;

    let mut in_origin = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[remote \"origin\"]" {
            in_origin = true;
            continue;
        }
        if trimmed.starts_with('[') {
            in_origin = false;
            continue;
        }
        if in_origin && trimmed.starts_with("url = ") {
            let url = trimmed.strip_prefix("url = ")?;
            return parse_github_owner_repo(url);
        }
    }
    None
}

/// URL에서 owner/repo 추출
fn parse_github_owner_repo(url: &str) -> Option<String> {
    let url = url.trim();
    // git@github.com:owner/repo.git
    if let Some(rest) = url.strip_prefix("git@github.com:") {
        let repo = rest.trim_end_matches(".git");
        return Some(repo.to_string());
    }
    // https://github.com/owner/repo.git
    if url.contains("github.com/") {
        let parts: Vec<&str> = url.split("github.com/").collect();
        if parts.len() == 2 {
            let repo = parts[1].trim_end_matches(".git");
            if repo.contains('/') {
                return Some(repo.to_string());
            }
        }
    }
    None
}

/// XP 추가 (서비스 레이어용)
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

/// GitHub 폴링 서비스 시작
pub async fn start_github_watcher(app: AppHandle) {
    // 시작 전 잠시 대기 (앱 초기화 완료 후)
    tokio::time::sleep(Duration::from_secs(5)).await;
    let mut interval = tokio::time::interval(Duration::from_secs(15));

    loop {
        interval.tick().await;

        let data = match storage::load(&app) {
            Ok(d) => d,
            Err(_) => continue,
        };

        let token = match &data.settings.github_token {
            Some(t) if !t.is_empty() => t.clone(),
            _ => continue,
        };

        // watched repos에서 GitHub remote 추출
        let repos = storage::get_watched_repos(&app);
        let mut github_repos: Vec<String> = Vec::new();

        for repo_path in &repos {
            if let Some(owner_repo) = extract_github_remote(repo_path) {
                if !github_repos.contains(&owner_repo) {
                    github_repos.push(owner_repo);
                }
            }
        }

        if github_repos.is_empty() {
            continue;
        }

        let client = reqwest::Client::new();

        for owner_repo in &github_repos {
            check_prs(&app, &client, &token, owner_repo).await;
            check_stars(&app, &client, &token, owner_repo).await;
        }
    }
}

/// PR 변동 체크
async fn check_prs(app: &AppHandle, client: &reqwest::Client, token: &str, owner_repo: &str) {
    let prs = match fetch_prs(client, token, owner_repo).await {
        Ok(p) => p,
        Err(_) => return,
    };

    let mut data = match storage::load(app) {
        Ok(d) => d,
        Err(_) => return,
    };

    let known_ids = data
        .github_state
        .last_pr_ids
        .entry(owner_repo.to_string())
        .or_default()
        .clone();
    let known_states = data
        .github_state
        .last_pr_states
        .entry(owner_repo.to_string())
        .or_default()
        .clone();

    let mut new_ids = known_ids.clone();
    let mut new_states = known_states.clone();
    let mut xp_actions: Vec<(u32, String)> = Vec::new();

    for pr in &prs {
        let is_merged = pr.merged_at.is_some();
        let state = if is_merged {
            "merged".to_string()
        } else {
            pr.state.clone()
        };

        if !known_ids.contains(&pr.number) {
            // 새 PR 발견
            new_ids.push(pr.number);
            new_states.insert(pr.number, state.clone());

            // 첫 로드가 아닌 경우에만 XP (known_ids가 비어있으면 첫 로드)
            if !known_ids.is_empty() && state == "open" {
                let _ = app.emit(
                    "github:pr-opened",
                    serde_json::json!({ "repo": owner_repo, "number": pr.number }),
                );
                xp_actions.push((20, "pr_open".to_string()));
            }
        } else {
            // 기존 PR 상태 변경
            let prev = known_states.get(&pr.number).cloned().unwrap_or_default();
            if prev != "merged" && is_merged {
                new_states.insert(pr.number, "merged".to_string());
                let _ = app.emit(
                    "github:pr-merged",
                    serde_json::json!({ "repo": owner_repo, "number": pr.number }),
                );
                xp_actions.push((30, "pr_merge".to_string()));
            } else {
                new_states.insert(pr.number, state);
            }
        }
    }

    // 상태 저장
    data.github_state
        .last_pr_ids
        .insert(owner_repo.to_string(), new_ids);
    data.github_state
        .last_pr_states
        .insert(owner_repo.to_string(), new_states);
    storage::save(app, &data).ok();

    // XP 추가 (save 후)
    for (amount, source) in xp_actions {
        let _ = add_xp_internal(app, amount, &source);
    }
}

/// Star 변동 체크
async fn check_stars(app: &AppHandle, client: &reqwest::Client, token: &str, owner_repo: &str) {
    let star_count = match fetch_star_count(client, token, owner_repo).await {
        Ok(c) => c,
        Err(_) => return,
    };

    let mut data = match storage::load(app) {
        Ok(d) => d,
        Err(_) => return,
    };

    let last = data.github_state.last_star_counts.get(owner_repo).copied();

    if let Some(prev) = last {
        if star_count > prev {
            let _ = app.emit("github:star-received", owner_repo.to_string());
        }
    }

    data.github_state
        .last_star_counts
        .insert(owner_repo.to_string(), star_count);
    storage::save(app, &data).ok();
}

async fn fetch_prs(
    client: &reqwest::Client,
    token: &str,
    owner_repo: &str,
) -> Result<Vec<GitHubPR>, String> {
    let url = format!(
        "https://api.github.com/repos/{}/pulls?state=all&per_page=30",
        owner_repo
    );
    let res = client
        .get(&url)
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .header(USER_AGENT, "commit-cat")
        .header(ACCEPT, "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| format!("PR fetch failed: {}", e))?;

    if !res.status().is_success() {
        return Err(format!("PR fetch status: {}", res.status()));
    }

    res.json()
        .await
        .map_err(|e| format!("PR parse failed: {}", e))
}

async fn fetch_star_count(
    client: &reqwest::Client,
    token: &str,
    owner_repo: &str,
) -> Result<u32, String> {
    let url = format!("https://api.github.com/repos/{}", owner_repo);
    let res = client
        .get(&url)
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .header(USER_AGENT, "commit-cat")
        .header(ACCEPT, "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| format!("Repo fetch failed: {}", e))?;

    if !res.status().is_success() {
        return Err(format!("Repo fetch status: {}", res.status()));
    }

    let repo: GitHubRepo = res
        .json()
        .await
        .map_err(|e| format!("Repo parse failed: {}", e))?;

    Ok(repo.stargazers_count)
}
