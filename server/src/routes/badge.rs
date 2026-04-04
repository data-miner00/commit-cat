use axum::extract::Path;
use axum::response::{IntoResponse, Response};
use axum::http::{header, StatusCode};

/// Fetch GitHub contributions count (last year)
async fn fetch_github_contributions(username: &str) -> Option<u32> {
    let url = format!("https://github.com/users/{}/contributions", username);
    let client = reqwest::Client::builder()
        .user_agent("CommitCat-Badge/1.0")
        .build()
        .ok()?;
    let html = client.get(&url).send().await.ok()?.text().await.ok()?;

    // HTML structure:
    //   <h2 ...>
    //     480
    //     contributions
    //       in the last year
    // Find line before "contributions" that contains just a number
    let lines: Vec<&str> = html.lines().map(|l| l.trim()).collect();
    for (i, line) in lines.iter().enumerate() {
        if *line == "contributions" || line.starts_with("contributions") {
            if i > 0 {
                let num_str = lines[i - 1].trim();
                let cleaned: String = num_str.chars().filter(|c| c.is_ascii_digit()).collect();
                if !cleaned.is_empty() {
                    return cleaned.parse().ok();
                }
            }
        }
    }
    None
}

pub async fn get_badge(
    Path(username): Path<String>,
) -> Response {
    let year = chrono::Local::now().format("%Y").to_string();
    let contributions = fetch_github_contributions(&username).await.unwrap_or(0);

    let svg = commit_cat_core::badge::generate_badge(contributions, &year, &username);

    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "image/svg+xml"),
            (header::CACHE_CONTROL, "public, max-age=3600"),
        ],
        svg,
    ).into_response()
}
