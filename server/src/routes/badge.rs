use axum::extract::Path;
use axum::response::{IntoResponse, Response};
use axum::http::{header, StatusCode};

/// Fetch GitHub contributions count for the current year
async fn fetch_github_contributions(username: &str) -> Option<u32> {
    let url = format!("https://github.com/users/{}/contributions", username);
    let client = reqwest::Client::builder()
        .user_agent("CommitCat-Badge/1.0")
        .build()
        .ok()?;
    let html = client.get(&url).send().await.ok()?.text().await.ok()?;

    // Parse: <h2 ...>295 contributions in the last year</h2>
    // or: <h2 ...>295 contributions in 2026</h2>
    for line in html.lines() {
        let trimmed = line.trim();
        if trimmed.contains("contributions") && trimmed.starts_with('<') {
            // Extract the number before "contributions"
            if let Some(idx) = trimmed.find("contributions") {
                let before = &trimmed[..idx];
                // Find the last '>' before the number
                if let Some(gt) = before.rfind('>') {
                    let num_str = before[gt + 1..].trim();
                    // Handle comma-separated numbers like "1,295"
                    let cleaned: String = num_str.chars().filter(|c| c.is_ascii_digit()).collect();
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
