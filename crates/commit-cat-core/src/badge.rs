// --- Animated badge (server feature: embeds cat sprites as base64) ---

#[cfg(feature = "server")]
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

#[cfg(feature = "server")]
const CAT_STAND: &[u8] = include_bytes!("../../../public/assets/cat/orange_stand.png");
#[cfg(feature = "server")]
const CAT_WALK: &[u8] = include_bytes!("../../../public/assets/cat/orange_walk.png");
#[cfg(feature = "server")]
const CAT_SIT: &[u8] = include_bytes!("../../../public/assets/cat/orange_sit.png");
#[cfg(feature = "server")]
const CAT_SLEEP: &[u8] = include_bytes!("../../../public/assets/cat/orange_sit2.png");
#[cfg(feature = "server")]
const CAT_PETTING: &[u8] = include_bytes!("../../../public/assets/cat/orange_petting.png");

/// Generate an SVG badge for GitHub README embedding
/// With "server" feature: animated cat with speech bubble
/// Without: simple text badge
pub fn generate_badge(level: u32, streak: u32, total_commits: u32, username: &str) -> String {
    #[cfg(feature = "server")]
    {
        generate_animated_badge(level, streak, total_commits, username)
    }
    #[cfg(not(feature = "server"))]
    {
        generate_simple_badge(level, streak, total_commits, username)
    }
}

// Simple text-only badge (desktop / non-server builds)
#[cfg(not(feature = "server"))]
fn generate_simple_badge(level: u32, streak: u32, total_commits: u32, username: &str) -> String {
    let streak_text = if streak > 0 { format!(" \u{00b7} {}d streak", streak) } else { String::new() };
    let label = format!("\u{1f431} {} \u{00b7} Lv.{}{} \u{00b7} {} commits", username, level, streak_text, total_commits);
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

/// Animated cat badge with speech bubble
#[cfg(feature = "server")]
fn generate_animated_badge(level: u32, streak: u32, total_commits: u32, username: &str) -> String {
    let stand = BASE64.encode(CAT_STAND);
    let walk = BASE64.encode(CAT_WALK);
    let sit = BASE64.encode(CAT_SIT);
    let sleep = BASE64.encode(CAT_SLEEP);
    let pet = BASE64.encode(CAT_PETTING);

    let streak_text = if streak > 0 {
        format!(" \u{00b7} {}d streak", streak)
    } else {
        String::new()
    };

    let info = format!("Lv.{} \u{00b7} {} commits{}", level, total_commits, streak_text);

    // Dynamic width: base 110 (cat area) + text width
    let username_px = username.len() as u32 * 9 + 30;
    let info_px = info.len() as u32 * 7 + 30;
    let text_w = username_px.max(info_px);
    let bubble_w = text_w.max(180);
    let width = bubble_w + 115; // cat area + padding

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="120" role="img" aria-label="CommitCat badge">
  <title>CommitCat - {username}</title>
  <defs>
    <style>
      .c-s {{ animation: s 12s ease-in-out infinite; }}
      .c-w {{ animation: w 12s ease-in-out infinite; }}
      .c-i {{ animation: i 12s ease-in-out infinite; }}
      .c-z {{ animation: z 12s ease-in-out infinite; }}
      .c-p {{ animation: p 12s ease-in-out infinite; }}
      .c-m {{ animation: m 12s ease-in-out infinite; }}
      @keyframes s {{
        0%,16.6%   {{ opacity:1 }}
        16.7%,33.2% {{ opacity:0 }}
        33.3%,41.6% {{ opacity:1 }}
        41.7%,100%  {{ opacity:0 }}
      }}
      @keyframes w {{
        0%,16.6%   {{ opacity:0 }}
        16.7%,33.2% {{ opacity:1 }}
        33.3%,41.6% {{ opacity:0 }}
        41.7%,58.2% {{ opacity:1 }}
        58.3%,100%  {{ opacity:0 }}
      }}
      @keyframes i {{
        0%,58.2%   {{ opacity:0 }}
        58.3%,74.9% {{ opacity:1 }}
        75%,100%    {{ opacity:0 }}
      }}
      @keyframes z {{
        0%,74.9%   {{ opacity:0 }}
        75%,87.4%  {{ opacity:1 }}
        87.5%,100% {{ opacity:0 }}
      }}
      @keyframes p {{
        0%,87.4%   {{ opacity:0 }}
        87.5%,100% {{ opacity:1 }}
      }}
      @keyframes m {{
        0%,16.6%   {{ transform:translateX(0) }}
        33.2%      {{ transform:translateX(18px) }}
        33.3%,41.6% {{ transform:translateX(18px) }}
        58.2%      {{ transform:translateX(0) }}
        58.3%,100% {{ transform:translateX(8px) }}
      }}
    </style>
  </defs>

  <!-- Background -->
  <rect width="{width}" height="120" rx="12" fill="#1a1a2e"/>

  <!-- Speech bubble -->
  <rect x="95" y="10" width="{bubble_w}" height="58" rx="10" fill="#2d2d44"/>
  <polygon points="110,68 96,82 124,68" fill="#2d2d44"/>

  <!-- Bubble text -->
  <g font-family="-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif" text-rendering="geometricPrecision">
    <text x="112" y="35" fill="#ffd700" font-size="14" font-weight="700">{username}</text>
    <text x="112" y="55" fill="#b0b0c8" font-size="12">{info}</text>
  </g>

  <!-- Animated cat -->
  <g class="c-m">
    <image class="c-s" x="12" y="58" width="75" height="54" href="data:image/png;base64,{stand}"/>
    <image class="c-w" x="12" y="58" width="75" height="54" href="data:image/png;base64,{walk}" opacity="0"/>
    <image class="c-i" x="12" y="58" width="75" height="54" href="data:image/png;base64,{sit}" opacity="0"/>
    <image class="c-z" x="12" y="58" width="75" height="54" href="data:image/png;base64,{sleep}" opacity="0"/>
    <image class="c-p" x="12" y="58" width="75" height="54" href="data:image/png;base64,{pet}" opacity="0"/>
  </g>
</svg>"##,
        width = width,
        bubble_w = bubble_w,
        username = username,
        info = info,
        stand = stand,
        walk = walk,
        sit = sit,
        sleep = sleep,
        pet = pet,
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
