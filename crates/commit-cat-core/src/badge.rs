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

    // Dynamic width: base 150 (cat area) + text width
    let username_px = username.len() as u32 * 10 + 40;
    let info_px = info.len() as u32 * 8 + 40;
    let text_w = username_px.max(info_px);
    let bubble_w = text_w.max(200);
    let width = bubble_w + 155; // cat area + padding

    // SMIL animation — proper state machine flow (16s total):
    //
    //   stand → walk → stand → sit → sleep → sit → petting → sit → stand → walk → stand
    //
    //   0.0-1.0s   stand           (x=0, still)
    //   1.0-3.0s   walk right      (x=0 → x=30)
    //   3.0-4.0s   stand           (x=30, still)
    //   4.0-5.0s   sit             (x=30, still)
    //   5.0-7.0s   sleep           (x=30, still)
    //   7.0-8.0s   sit             (x=30, still)
    //   8.0-9.0s   petting         (x=30, still)
    //   9.0-10.0s  sit             (x=30, still)
    //  10.0-11.0s  stand           (x=30, still)
    //  11.0-13.0s  walk left       (x=30 → x=0)
    //  13.0-14.0s  stand           (x=0, still — loops)
    //
    // keyTimes (14s):
    //  0/14=0.000   1/14=0.071   3/14=0.214   4/14=0.286
    //  5/14=0.357   7/14=0.500   8/14=0.571   9/14=0.643
    // 10/14=0.714  11/14=0.786  13/14=0.929  14/14=1.000
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="{width}" height="150" role="img" aria-label="CommitCat badge">
  <title>CommitCat - {username}</title>

  <!-- Background -->
  <rect width="{width}" height="150" rx="14" fill="#7FD17F"/>

  <!-- Speech bubble -->
  <rect x="130" y="12" width="{bubble_w}" height="62" rx="12" fill="#fff" opacity="0.9"/>
  <polygon points="146,74 128,90 160,74" fill="#fff" opacity="0.9"/>

  <!-- Bubble text -->
  <g font-family="-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif" text-rendering="geometricPrecision">
    <text x="148" y="40" fill="#2a6e2a" font-size="16" font-weight="700">{username}</text>
    <text x="148" y="62" fill="#555" font-size="13">{info}</text>
  </g>

  <!-- Animated cat (SMIL) — movement only during walk phases -->
  <g>
    <animateTransform attributeName="transform" type="translate" dur="14s" repeatCount="indefinite"
      values="0,0; 0,0; 30,0; 30,0; 30,0; 30,0; 30,0; 30,0; 30,0; 30,0; 30,0; 0,0; 0,0"
      keyTimes="0; 0.071; 0.214; 0.286; 0.357; 0.500; 0.571; 0.643; 0.714; 0.786; 0.929; 0.930; 1"
      calcMode="linear"/>

    <!-- stand: 0-1s, 3-4s, 10-11s, 13-14s -->
    <image x="10" y="55" width="117" height="85" href="data:image/png;base64,{stand}" opacity="0">
      <animate attributeName="opacity" dur="14s" repeatCount="indefinite"
        values="1;1; 0;0; 1;1; 0;0; 0;0; 0;0; 0;0; 1;1; 0;0; 1;1"
        keyTimes="0;0.070; 0.071;0.213; 0.214;0.285; 0.286;0.356; 0.357;0.499; 0.500;0.570; 0.571;0.713; 0.714;0.785; 0.786;0.928; 0.929;1"/>
    </image>

    <!-- walk: 1-3s, 11-13s -->
    <image x="10" y="55" width="106" height="77" href="data:image/png;base64,{walk}" opacity="0">
      <animate attributeName="opacity" dur="14s" repeatCount="indefinite"
        values="0;0; 1;1; 0;0; 1;1; 0"
        keyTimes="0;0.070; 0.071;0.213; 0.214;0.785; 0.786;0.928; 1"/>
    </image>

    <!-- sit: 4-5s, 7-8s, 9-10s -->
    <image x="10" y="55" width="110" height="80" href="data:image/png;base64,{sit}" opacity="0">
      <animate attributeName="opacity" dur="14s" repeatCount="indefinite"
        values="0;0; 1;1; 0;0; 1;1; 0;0; 1;1; 0;0"
        keyTimes="0;0.285; 0.286;0.356; 0.357;0.499; 0.500;0.570; 0.571;0.642; 0.643;0.713; 0.714;1"/>
    </image>

    <!-- sleep: 5-7s -->
    <image x="10" y="55" width="110" height="80" href="data:image/png;base64,{sleep}" opacity="0">
      <animate attributeName="opacity" dur="14s" repeatCount="indefinite"
        values="0;0; 1;1; 0;0"
        keyTimes="0;0.356; 0.357;0.499; 0.500;1"/>
    </image>

    <!-- petting: 8-9s -->
    <image x="10" y="55" width="110" height="80" href="data:image/png;base64,{pet}" opacity="0">
      <animate attributeName="opacity" dur="14s" repeatCount="indefinite"
        values="0;0; 1;1; 0;0"
        keyTimes="0;0.570; 0.571;0.642; 0.643;1"/>
    </image>
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
