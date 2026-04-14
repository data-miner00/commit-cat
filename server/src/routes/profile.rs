use axum::extract::{Path, State};
use axum::response::{Html, IntoResponse};
use axum::Json;
use crate::AppState;

async fn fetch_user_rank(state: &AppState, level: i32, exp: i32) -> Option<i64> {
    sqlx::query_scalar::<_, i64>(
        "SELECT 1 + COUNT(*) FROM user_data
         WHERE level > ? OR (level = ? AND exp > ?)"
    )
    .bind(level)
    .bind(level)
    .bind(exp)
    .fetch_one(&state.db)
    .await
    .ok()
}

async fn fetch_recent_activity(
    state: &AppState,
    user_id: &str,
    limit: i64,
) -> Vec<(String, i32, String)> {
    sqlx::query_as::<_, (String, i32, String)>(
        "SELECT event_type, xp, occurred_at
         FROM sync_events
         WHERE user_id = ?
         ORDER BY occurred_at DESC
         LIMIT ?"
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default()
}

/// Activity history endpoint (last 30 sync events)
pub async fn get_profile_activity(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Json<serde_json::Value> {
    let user_id: Option<String> = sqlx::query_scalar(
        "SELECT id FROM users WHERE github_username = ?"
    )
    .bind(&username)
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten();

    let Some(uid) = user_id else {
        return Json(serde_json::json!({"error": "User not found", "username": username}));
    };

    let events = fetch_recent_activity(&state, &uid, 30).await;
    let entries: Vec<serde_json::Value> = events.into_iter().map(|(event_type, xp, occurred_at)| {
        serde_json::json!({
            "type": event_type,
            "xp": xp,
            "occurredAt": occurred_at,
        })
    }).collect();

    Json(serde_json::json!({
        "username": username,
        "events": entries,
    }))
}

/// Public profile HTML page
pub async fn get_profile_page(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> impl IntoResponse {
    let row = sqlx::query_as::<_, (String, i32, i32, i32, i32, i32, Option<String>, String, Option<String>)>(
        "SELECT u.id, ud.level, ud.exp, ud.total_commits, ud.current_streak, ud.longest_streak,
                ud.current_hat, ud.unlocked_hats, u.github_avatar_url
         FROM users u
         JOIN user_data ud ON u.id = ud.user_id
         WHERE u.github_username = ?"
    )
    .bind(&username)
    .fetch_optional(&state.db)
    .await;

    match row {
        Ok(Some((user_id, level, exp, commits, streak, longest, hat, hats_json, avatar))) => {
            let unlocked: Vec<String> = serde_json::from_str(&hats_json).unwrap_or_default();
            let avatar_url = avatar.unwrap_or_default();
            let hat_display = hat.unwrap_or_else(|| "none".to_string());
            let next_level_xp = level * 100;
            let progress_pct = if next_level_xp > 0 { (exp as f64 / next_level_xp as f64 * 100.0).min(100.0) } else { 0.0 };

            let rank = fetch_user_rank(&state, level, exp).await;
            let rank_display = match rank {
                Some(r) => format!("#{}", r),
                None => "-".to_string(),
            };

            let recent = fetch_recent_activity(&state, &user_id, 10).await;
            let activity_html: String = recent.iter().map(|(event_type, xp, occurred_at)| {
                let icon = match event_type.as_str() {
                    "commit" => "📝",
                    "push" => "⬆️",
                    "pomodoro" => "🍅",
                    "coding_time" => "💻",
                    "pr_open" | "pr_opened" => "🔀",
                    "pr_merge" | "pr_merged" => "✅",
                    "build_success" => "🔨",
                    "build_fail" => "❌",
                    _ => "✨",
                };
                let date = occurred_at.split('T').next().unwrap_or(occurred_at);
                format!(r#"<li><span class="evt-icon">{icon}</span><span class="evt-type">{event_type}</span><span class="evt-xp">+{xp} XP</span><span class="evt-date">{date}</span></li>"#)
            }).collect();

            let badge_url = format!("/badge/{}", username);

            Html(format!(r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{username} — CommitCat</title>
<meta property="og:title" content="{username} — CommitCat">
<meta property="og:description" content="Lv.{level} · {commits} commits · {streak}d streak">
<meta property="og:image" content="{avatar_url}">
<style>
  * {{ margin: 0; padding: 0; box-sizing: border-box; }}
  body {{
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    background: #0d1117;
    color: #e6edf3;
    min-height: 100vh;
    display: flex;
    justify-content: center;
    padding: 40px 20px;
  }}
  .profile {{
    max-width: 560px;
    width: 100%;
  }}
  .header {{
    display: flex;
    align-items: center;
    gap: 20px;
    margin-bottom: 32px;
  }}
  .avatar {{
    width: 80px;
    height: 80px;
    border-radius: 50%;
    border: 3px solid #ffd700;
  }}
  .avatar-placeholder {{
    width: 80px;
    height: 80px;
    border-radius: 50%;
    background: #21262d;
    border: 3px solid #ffd700;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 36px;
  }}
  .name {{
    font-size: 28px;
    font-weight: 700;
  }}
  .level-tag {{
    display: inline-block;
    background: #1a1a2e;
    color: #ffd700;
    padding: 4px 12px;
    border-radius: 12px;
    font-size: 14px;
    font-weight: 600;
    margin-top: 4px;
  }}
  .stats {{
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
    margin-bottom: 24px;
  }}
  .stat-card {{
    background: #161b22;
    border: 1px solid #30363d;
    border-radius: 12px;
    padding: 16px;
  }}
  .stat-label {{
    font-size: 12px;
    color: #8b949e;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 4px;
  }}
  .stat-value {{
    font-size: 24px;
    font-weight: 700;
  }}
  .stat-value.gold {{ color: #ffd700; }}
  .stat-value.fire {{ color: #ff6b35; }}
  .stat-value.green {{ color: #3fb950; }}
  .stat-value.purple {{ color: #bc8cff; }}
  .xp-section {{
    background: #161b22;
    border: 1px solid #30363d;
    border-radius: 12px;
    padding: 16px;
    margin-bottom: 24px;
  }}
  .xp-header {{
    display: flex;
    justify-content: space-between;
    font-size: 13px;
    color: #8b949e;
    margin-bottom: 8px;
  }}
  .xp-bar {{
    height: 8px;
    background: #21262d;
    border-radius: 4px;
    overflow: hidden;
  }}
  .xp-fill {{
    height: 100%;
    background: linear-gradient(90deg, #ffd700, #ffaa00);
    border-radius: 4px;
    transition: width 0.3s;
  }}
  .hats {{
    background: #161b22;
    border: 1px solid #30363d;
    border-radius: 12px;
    padding: 16px;
    margin-bottom: 24px;
  }}
  .hats h3 {{
    font-size: 14px;
    color: #8b949e;
    margin-bottom: 8px;
  }}
  .hat-list {{
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }}
  .hat-tag {{
    background: #21262d;
    padding: 4px 10px;
    border-radius: 8px;
    font-size: 13px;
  }}
  .hat-tag.equipped {{
    background: #1a1a2e;
    border: 1px solid #ffd700;
    color: #ffd700;
  }}
  .badge-section {{
    margin-bottom: 24px;
  }}
  .badge-section img {{
    max-width: 100%;
  }}
  .activity {{
    background: #161b22;
    border: 1px solid #30363d;
    border-radius: 12px;
    padding: 16px;
    margin-bottom: 24px;
  }}
  .activity h3 {{
    font-size: 14px;
    color: #8b949e;
    margin-bottom: 12px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }}
  .activity ul {{ list-style: none; }}
  .activity li {{
    display: grid;
    grid-template-columns: 24px 1fr auto auto;
    align-items: center;
    gap: 12px;
    padding: 8px 0;
    font-size: 13px;
    border-bottom: 1px solid #21262d;
  }}
  .activity li:last-child {{ border-bottom: none; }}
  .evt-icon {{ font-size: 16px; }}
  .evt-type {{ color: #e6edf3; }}
  .evt-xp {{ color: #3fb950; font-variant-numeric: tabular-nums; }}
  .evt-date {{ color: #8b949e; font-size: 11px; font-variant-numeric: tabular-nums; }}
  .leaderboard-link {{
    display: inline-block;
    margin-left: 8px;
    background: #1a1a2e;
    border: 1px solid #ffd700;
    color: #ffd700;
    padding: 4px 10px;
    border-radius: 12px;
    font-size: 13px;
    font-weight: 600;
    text-decoration: none;
  }}
  .leaderboard-link:hover {{ background: #ffd700; color: #1a1a2e; }}
  .footer {{
    text-align: center;
    color: #484f58;
    font-size: 12px;
    margin-top: 32px;
  }}
  .footer a {{
    color: #58a6ff;
    text-decoration: none;
  }}
</style>
</head>
<body>
<div class="profile">
  <div class="header">
    {avatar_html}
    <div>
      <div class="name">{username}</div>
      <span class="level-tag">Lv.{level}</span>
      <a class="leaderboard-link" href="/leaderboard">Rank {rank_display}</a>
    </div>
  </div>

  <div class="stats">
    <div class="stat-card">
      <div class="stat-label">Total Commits</div>
      <div class="stat-value green">{commits}</div>
    </div>
    <div class="stat-card">
      <div class="stat-label">Current Streak</div>
      <div class="stat-value fire">{streak}d</div>
    </div>
    <div class="stat-card">
      <div class="stat-label">Longest Streak</div>
      <div class="stat-value purple">{longest}d</div>
    </div>
    <div class="stat-card">
      <div class="stat-label">Current Hat</div>
      <div class="stat-value gold">{hat_display}</div>
    </div>
  </div>

  <div class="xp-section">
    <div class="xp-header">
      <span>XP Progress</span>
      <span>{exp} / {next_level_xp} XP</span>
    </div>
    <div class="xp-bar">
      <div class="xp-fill" style="width:{progress_pct:.1}%"></div>
    </div>
  </div>

  {hats_section}

  {activity_section}

  <div class="badge-section">
    <img src="{badge_url}" alt="CommitCat Badge">
  </div>

  <div class="footer">
    Powered by <a href="https://github.com/eunseo9311/commit-cat">CommitCat</a>
  </div>
</div>
</body>
</html>"##,
                username = username,
                level = level,
                exp = exp,
                commits = commits,
                streak = streak,
                longest = longest,
                hat_display = hat_display,
                next_level_xp = next_level_xp,
                progress_pct = progress_pct,
                avatar_url = avatar_url,
                rank_display = rank_display,
                activity_section = if activity_html.is_empty() {
                    String::new()
                } else {
                    format!(r#"<div class="activity"><h3>Recent Activity</h3><ul>{}</ul></div>"#, activity_html)
                },
                avatar_html = if avatar_url.is_empty() {
                    r#"<div class="avatar-placeholder">🐱</div>"#.to_string()
                } else {
                    format!(r#"<img class="avatar" src="{}" alt="{}">"#, avatar_url, username)
                },
                badge_url = badge_url,
                hats_section = if unlocked.is_empty() {
                    String::new()
                } else {
                    let tags: Vec<String> = unlocked.iter().map(|h| {
                        if Some(h.as_str()) == Some(hat_display.as_str()) && hat_display != "none" {
                            format!(r#"<span class="hat-tag equipped">{}</span>"#, h)
                        } else {
                            format!(r#"<span class="hat-tag">{}</span>"#, h)
                        }
                    }).collect();
                    format!(r#"<div class="hats"><h3>Unlocked Items</h3><div class="hat-list">{}</div></div>"#, tags.join(""))
                },
            )).into_response()
        }
        _ => {
            Html(format!(r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{username} — CommitCat</title>
<style>
  body {{
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    background: #0d1117;
    color: #e6edf3;
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
  }}
  .not-found {{
    text-align: center;
  }}
  .not-found h1 {{
    font-size: 48px;
    margin-bottom: 12px;
  }}
  .not-found p {{
    color: #8b949e;
    font-size: 16px;
  }}
  .not-found a {{
    color: #58a6ff;
    text-decoration: none;
  }}
</style>
</head>
<body>
<div class="not-found">
  <h1>🐱</h1>
  <p><strong>{username}</strong> hasn't joined CommitCat yet.</p>
  <p style="margin-top:12px"><a href="https://github.com/eunseo9311/commit-cat">Get CommitCat →</a></p>
</div>
</body>
</html>"##, username = username)).into_response()
        }
    }
}

pub async fn get_profile(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Json<serde_json::Value> {
    let row = sqlx::query_as::<_, (i32, i32, i32, i32, i32, Option<String>, String, Option<String>)>(
        "SELECT ud.level, ud.exp, ud.total_commits, ud.current_streak, ud.longest_streak,
                ud.current_hat, ud.unlocked_hats, u.github_avatar_url
         FROM users u
         JOIN user_data ud ON u.id = ud.user_id
         WHERE u.github_username = ?"
    )
    .bind(&username)
    .fetch_optional(&state.db)
    .await;

    match row {
        Ok(Some((level, exp, commits, streak, longest, hat, hats_json, avatar))) => {
            let unlocked: Vec<String> = serde_json::from_str(&hats_json).unwrap_or_default();
            Json(serde_json::json!({
                "username": username,
                "avatarUrl": avatar,
                "level": level,
                "exp": exp,
                "totalCommits": commits,
                "currentStreak": streak,
                "longestStreak": longest,
                "currentHat": hat,
                "unlockedHats": unlocked,
            }))
        }
        _ => Json(serde_json::json!({
            "error": "User not found",
            "username": username,
        })),
    }
}
