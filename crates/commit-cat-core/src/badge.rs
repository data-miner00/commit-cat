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
    let info_line = format!("Lv.{} \u{00b7} {} commits{}", level, total_commits, streak_text);

    // Layout: [cat roaming area] [pill badge on right]
    let cat_w: u32 = 100;
    let cat_h: u32 = 72;
    let width: u32 = 420;
    let height: u32 = 120;
    let cat_y = height - cat_h - 6;

    // Pill badge sizing
    let uname_px = username.len() as u32 * 9 + 36;
    let info_px = info_line.len() as u32 * 7 + 36;
    let pill_w = uname_px.max(info_px).max(170);
    let pill_h: u32 = 56;
    let pill_x = width - pill_w - 16;
    let pill_y: u32 = (height - pill_h) / 2;
    let pill_r = pill_h / 2;

    // Flip transform for left-facing sprites: translate(2*x + w, 0) scale(-1, 1)
    let flip_tx = 2 * 8 + cat_w;

    // SMIL — proper state machine flow (18s, no gaps, no teleport):
    //
    //  0-1.5s    stand→      x=0    (facing right, still)
    //  1.5-5s    walk→       x→70   (facing right, moving right)
    //  5-6.5s    stand→      x=70   (facing right, still)
    //  6.5-7.5s  sit         x=70   (still)
    //  7.5-9.5s  sleep       x=70   (still)
    //  9.5-10.5s sit         x=70   (still)
    // 10.5-11.5s petting     x=70   (still)
    // 11.5-12.5s sit         x=70   (still)
    // 12.5-13.5s stand←      x=70   (facing left, still)
    // 13.5-17s   walk←       x→0    (facing left, moving left)
    // 17-18s     stand→      x=0    (facing right, still — loops)
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="{width}" height="{height}" role="img" aria-label="CommitCat badge">
  <title>CommitCat - {username}</title>

  <!-- Background -->
  <rect width="{width}" height="{height}" rx="14" fill="#7FD17F"/>

  <!-- Pill badge (no tail) -->
  <rect x="{pill_x}" y="{pill_y}" width="{pill_w}" height="{pill_h}" rx="{pill_r}" fill="#fff" opacity="0.92"/>
  <g font-family="-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif" text-rendering="geometricPrecision" text-anchor="middle">
    <text x="{text_cx}" y="{uname_y}" fill="#2a6e2a" font-size="14" font-weight="700">{username}</text>
    <text x="{text_cx}" y="{info_y}" fill="#666" font-size="11">{info_line}</text>
  </g>

  <!-- Animated cat (SMIL) -->
  <g>
    <!-- Position: smooth move during walk only, still otherwise -->
    <animateTransform attributeName="transform" type="translate" dur="18s" repeatCount="indefinite"
      values="0,0; 0,0; 70,0; 70,0; 70,0; 70,0; 70,0; 70,0; 70,0; 70,0; 0,0; 0,0"
      keyTimes="0; 0.083; 0.278; 0.361; 0.417; 0.528; 0.583; 0.639; 0.694; 0.750; 0.944; 1"/>

    <!-- stand→: 0-1.5s, 5-6.5s, 17-18s -->
    <image x="8" y="{cat_y}" width="{cat_w}" height="{cat_h}" href="data:image/png;base64,{stand}" opacity="0">
      <animate attributeName="opacity" dur="18s" repeatCount="indefinite"
        values="1;1; 0;0; 1;1; 0;0; 1;1"
        keyTimes="0;0.082; 0.083;0.277; 0.278;0.360; 0.361;0.943; 0.944;1"/>
    </image>

    <!-- walk→: 1.5-5s -->
    <image x="8" y="{cat_y}" width="{cat_w}" height="{cat_h}" href="data:image/png;base64,{walk}" opacity="0">
      <animate attributeName="opacity" dur="18s" repeatCount="indefinite"
        values="0;0; 1;1; 0;0"
        keyTimes="0;0.082; 0.083;0.277; 0.278;1"/>
    </image>

    <!-- stand←: 12.5-13.5s (flipped) -->
    <g transform="translate({flip_tx},0) scale(-1,1)">
      <image x="8" y="{cat_y}" width="{cat_w}" height="{cat_h}" href="data:image/png;base64,{stand}" opacity="0">
        <animate attributeName="opacity" dur="18s" repeatCount="indefinite"
          values="0;0; 1;1; 0;0"
          keyTimes="0;0.693; 0.694;0.749; 0.750;1"/>
      </image>
    </g>

    <!-- walk←: 13.5-17s (flipped) -->
    <g transform="translate({flip_tx},0) scale(-1,1)">
      <image x="8" y="{cat_y}" width="{cat_w}" height="{cat_h}" href="data:image/png;base64,{walk}" opacity="0">
        <animate attributeName="opacity" dur="18s" repeatCount="indefinite"
          values="0;0; 1;1; 0;0"
          keyTimes="0;0.749; 0.750;0.943; 0.944;1"/>
      </image>
    </g>

    <!-- sit: 6.5-7.5s, 9.5-10.5s, 11.5-12.5s -->
    <image x="8" y="{cat_y}" width="{cat_w}" height="{cat_h}" href="data:image/png;base64,{sit}" opacity="0">
      <animate attributeName="opacity" dur="18s" repeatCount="indefinite"
        values="0;0; 1;1; 0;0; 1;1; 0;0; 1;1; 0;0"
        keyTimes="0;0.360; 0.361;0.416; 0.417;0.527; 0.528;0.582; 0.583;0.638; 0.639;0.693; 0.694;1"/>
    </image>

    <!-- sleep: 7.5-9.5s -->
    <image x="8" y="{cat_y}" width="{cat_w}" height="{cat_h}" href="data:image/png;base64,{sleep}" opacity="0">
      <animate attributeName="opacity" dur="18s" repeatCount="indefinite"
        values="0;0; 1;1; 0;0"
        keyTimes="0;0.416; 0.417;0.527; 0.528;1"/>
    </image>

    <!-- petting: 10.5-11.5s -->
    <image x="8" y="{cat_y}" width="{cat_w}" height="{cat_h}" href="data:image/png;base64,{pet}" opacity="0">
      <animate attributeName="opacity" dur="18s" repeatCount="indefinite"
        values="0;0; 1;1; 0;0"
        keyTimes="0;0.582; 0.583;0.638; 0.639;1"/>
    </image>
  </g>
</svg>"##,
        width = width,
        height = height,
        pill_x = pill_x,
        pill_y = pill_y,
        pill_w = pill_w,
        pill_h = pill_h,
        pill_r = pill_r,
        text_cx = pill_x + pill_w / 2,
        uname_y = pill_y + 22,
        info_y = pill_y + 40,
        cat_y = cat_y,
        cat_w = cat_w,
        cat_h = cat_h,
        flip_tx = flip_tx,
        username = username,
        info_line = info_line,
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
