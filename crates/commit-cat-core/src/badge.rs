/// Generate an SVG badge for GitHub README embedding
pub fn generate_badge(level: u32, streak: u32, total_commits: u32, username: &str) -> String {
    let streak_text = if streak > 0 { format!(" · {}d streak", streak) } else { String::new() };
    let label = format!("🐱 {} · Lv.{}{} · {} commits", username, level, streak_text, total_commits);
    let text_width = label.len() as u32 * 7 + 20;
    let width = text_width.max(200);

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="28" role="img" aria-label="CommitCat badge">
  <title>{label}</title>
  <linearGradient id="bg" x2="0" y2="100%">
    <stop offset="0" stop-color="#2a2a3a"/>
    <stop offset="1" stop-color="#1a1a2e"/>
  </linearGradient>
  <clipPath id="r">
    <rect width="{width}" height="28" rx="5" fill="#fff"/>
  </clipPath>
  <g clip-path="url(#r)">
    <rect width="{width}" height="28" fill="url(#bg)"/>
  </g>
  <g fill="#fff" text-anchor="start" font-family="monospace" text-rendering="geometricPrecision" font-size="12">
    <text x="10" y="19" fill="#ffd700" font-weight="600">{label}</text>
  </g>
</svg>"##,
        width = width,
        label = label,
    )
}

/// Generate a simple level badge
pub fn generate_level_badge(level: u32) -> String {
    let width = 80;
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="24" role="img">
  <rect width="{width}" height="24" rx="4" fill="#1a1a2e"/>
  <text x="{tx}" y="16" text-anchor="middle" fill="#ffd700" font-family="monospace" font-size="12" font-weight="600">🐱 Lv.{level}</text>
</svg>"##,
        width = width,
        tx = width / 2,
        level = level,
    )
}

/// Generate a streak badge
pub fn generate_streak_badge(streak: u32) -> String {
    let text = format!("🔥 {}d streak", streak);
    let width = text.len() as u32 * 7 + 16;
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="24" role="img">
  <rect width="{width}" height="24" rx="4" fill="#1a1a2e"/>
  <text x="{tx}" y="16" text-anchor="middle" fill="#ff6b35" font-family="monospace" font-size="12" font-weight="600">{text}</text>
</svg>"##,
        width = width,
        tx = width / 2,
        text = text,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn badge_contains_username() {
        let svg = generate_badge(10, 5, 100, "testuser");
        assert!(svg.contains("testuser"));
        assert!(svg.contains("Lv.10"));
        assert!(svg.contains("5d streak"));
        assert!(svg.contains("100 commits"));
    }

    #[test]
    fn badge_no_streak_when_zero() {
        let svg = generate_badge(1, 0, 5, "newuser");
        assert!(!svg.contains("streak"));
    }

    #[test]
    fn level_badge_contains_level() {
        let svg = generate_level_badge(42);
        assert!(svg.contains("Lv.42"));
    }

    #[test]
    fn streak_badge_contains_days() {
        let svg = generate_streak_badge(30);
        assert!(svg.contains("30d streak"));
    }
}
