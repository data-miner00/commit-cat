use axum::extract::{Query, State};
use axum::response::{Html, IntoResponse};
use axum::Json;
use serde::Deserialize;
use crate::AppState;

#[derive(Deserialize)]
pub struct LeaderboardQuery {
    #[serde(default = "default_sort")]
    pub sort: String,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_sort() -> String { "level".to_string() }
fn default_limit() -> u32 { 50 }

pub async fn get_leaderboard_json(
    State(state): State<AppState>,
    Query(q): Query<LeaderboardQuery>,
) -> Json<serde_json::Value> {
    let (rows, sort) = fetch_leaderboard(&state, &q.sort, q.limit).await;

    let entries: Vec<serde_json::Value> = rows.into_iter().enumerate().map(|(i, r)| {
        serde_json::json!({
            "rank": i + 1,
            "username": r.username,
            "avatarUrl": r.avatar_url,
            "level": r.level,
            "totalCommits": r.total_commits,
            "currentStreak": r.current_streak,
            "longestStreak": r.longest_streak,
        })
    }).collect();

    Json(serde_json::json!({
        "sort": sort,
        "entries": entries,
    }))
}

pub async fn get_leaderboard_page(
    State(state): State<AppState>,
    Query(q): Query<LeaderboardQuery>,
) -> impl IntoResponse {
    let (rows, sort) = fetch_leaderboard(&state, &q.sort, q.limit.min(100)).await;

    let rows_html: String = rows.into_iter().enumerate().map(|(i, r)| {
        let rank = i + 1;
        let medal = match rank {
            1 => "🥇",
            2 => "🥈",
            3 => "🥉",
            _ => "",
        };
        let avatar = r.avatar_url.unwrap_or_default();
        let avatar_html = if avatar.is_empty() {
            r#"<div class="avatar-placeholder">🐱</div>"#.to_string()
        } else {
            format!(r#"<img class="avatar" src="{}" alt="">"#, avatar)
        };
        format!(r#"
          <tr>
            <td class="rank">{medal} {rank}</td>
            <td class="user">
              <a href="/profile/{username}" class="user-link">
                {avatar_html}
                <span>{username}</span>
              </a>
            </td>
            <td class="num level">{level}</td>
            <td class="num commits">{commits}</td>
            <td class="num streak">{streak}d</td>
          </tr>
        "#,
            medal = medal,
            rank = rank,
            username = r.username,
            avatar_html = avatar_html,
            level = r.level,
            commits = r.total_commits,
            streak = r.current_streak,
        )
    }).collect();

    let (level_class, commits_class, streak_class) = match sort.as_str() {
        "commits" => ("", "active", ""),
        "streak" => ("", "", "active"),
        _ => ("active", "", ""),
    };

    Html(format!(r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Leaderboard — CommitCat</title>
<style>
  * {{ margin: 0; padding: 0; box-sizing: border-box; }}
  body {{
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    background: #0d1117;
    color: #e6edf3;
    min-height: 100vh;
    padding: 40px 20px;
  }}
  .wrap {{ max-width: 720px; margin: 0 auto; }}
  h1 {{ font-size: 32px; margin-bottom: 8px; }}
  .subtitle {{ color: #8b949e; font-size: 14px; margin-bottom: 24px; }}
  .tabs {{ display: flex; gap: 8px; margin-bottom: 16px; }}
  .tab {{
    padding: 8px 16px;
    background: #161b22;
    border: 1px solid #30363d;
    border-radius: 8px;
    color: #8b949e;
    text-decoration: none;
    font-size: 14px;
  }}
  .tab.active {{ background: #1a1a2e; border-color: #ffd700; color: #ffd700; }}
  table {{
    width: 100%;
    background: #161b22;
    border: 1px solid #30363d;
    border-radius: 12px;
    border-collapse: separate;
    border-spacing: 0;
    overflow: hidden;
  }}
  th, td {{ padding: 12px 16px; text-align: left; }}
  th {{
    background: #1a1a2e;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: #8b949e;
    font-weight: 600;
  }}
  tr + tr td {{ border-top: 1px solid #30363d; }}
  td.rank {{ color: #ffd700; font-weight: 700; width: 80px; }}
  td.num {{ text-align: right; font-variant-numeric: tabular-nums; }}
  td.level {{ color: #ffd700; }}
  td.commits {{ color: #3fb950; }}
  td.streak {{ color: #ff6b35; }}
  .user-link {{
    display: flex;
    align-items: center;
    gap: 10px;
    color: #e6edf3;
    text-decoration: none;
  }}
  .user-link:hover {{ color: #58a6ff; }}
  .avatar, .avatar-placeholder {{
    width: 28px;
    height: 28px;
    border-radius: 50%;
  }}
  .avatar-placeholder {{
    background: #21262d;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 14px;
  }}
  .empty {{
    text-align: center;
    padding: 60px 20px;
    color: #8b949e;
  }}
  .footer {{
    text-align: center;
    color: #484f58;
    font-size: 12px;
    margin-top: 32px;
  }}
  .footer a {{ color: #58a6ff; text-decoration: none; }}
</style>
</head>
<body>
<div class="wrap">
  <h1>🏆 Leaderboard</h1>
  <p class="subtitle">Top CommitCat users ranked by {sort}</p>

  <div class="tabs">
    <a class="tab {level_class}" href="?sort=level">By Level</a>
    <a class="tab {commits_class}" href="?sort=commits">By Commits</a>
    <a class="tab {streak_class}" href="?sort=streak">By Streak</a>
  </div>

  {body}

  <div class="footer">
    <a href="/">← Home</a> · <a href="https://github.com/eunseo9311/commit-cat">GitHub</a>
  </div>
</div>
</body>
</html>"##,
        sort = sort,
        level_class = level_class,
        commits_class = commits_class,
        streak_class = streak_class,
        body = if rows_html.is_empty() {
            r#"<div class="empty">No users yet. Be the first to sign up!</div>"#.to_string()
        } else {
            format!(r#"<table>
      <thead>
        <tr><th>Rank</th><th>User</th><th style="text-align:right">Level</th><th style="text-align:right">Commits</th><th style="text-align:right">Streak</th></tr>
      </thead>
      <tbody>{}</tbody>
    </table>"#, rows_html)
        },
    )).into_response()
}

struct Row {
    username: String,
    avatar_url: Option<String>,
    level: i32,
    total_commits: i32,
    current_streak: i32,
    #[allow(dead_code)]
    longest_streak: i32,
}

async fn fetch_leaderboard(state: &AppState, sort: &str, limit: u32) -> (Vec<Row>, String) {
    let (order_by, normalized) = match sort {
        "commits" => ("ud.total_commits DESC, ud.level DESC", "commits".to_string()),
        "streak" => ("ud.current_streak DESC, ud.longest_streak DESC", "streak".to_string()),
        _ => ("ud.level DESC, ud.exp DESC", "level".to_string()),
    };

    let sql = format!(
        "SELECT u.github_username, u.github_avatar_url,
                ud.level, ud.total_commits, ud.current_streak, ud.longest_streak
         FROM users u
         JOIN user_data ud ON u.id = ud.user_id
         ORDER BY {order_by}
         LIMIT ?"
    );

    let rows = sqlx::query_as::<_, (String, Option<String>, i32, i32, i32, i32)>(&sql)
        .bind(limit as i64)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|(username, avatar_url, level, total_commits, current_streak, longest_streak)| Row {
            username,
            avatar_url,
            level,
            total_commits,
            current_streak,
            longest_streak,
        })
        .collect();

    (rows, normalized)
}
