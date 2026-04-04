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
/// With "server" feature: animated cat walking around
/// Without: simple text badge
pub fn generate_badge(contributions: u32, year: &str, username: &str) -> String {
    #[cfg(feature = "server")]
    {
        generate_animated_badge(contributions, year, username)
    }
    #[cfg(not(feature = "server"))]
    {
        generate_simple_badge(contributions, year, username)
    }
}

// Simple text-only badge (desktop / non-server builds)
#[cfg(not(feature = "server"))]
fn generate_simple_badge(contributions: u32, year: &str, username: &str) -> String {
    let label = format!("\u{1f431} {} \u{00b7} {} contributions in {}", username, contributions, year);
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
fn generate_animated_badge(contributions: u32, year: &str, username: &str) -> String {
    let stand = BASE64.encode(CAT_STAND);
    let walk = BASE64.encode(CAT_WALK);
    let sit = BASE64.encode(CAT_SIT);
    let sleep = BASE64.encode(CAT_SLEEP);
    let pet = BASE64.encode(CAT_PETTING);

    let info_line = format!("{} contributions in {}", contributions, year);

    // Sprite faces LEFT by default. Flip (scaleX -1) for RIGHT.
    // Layout: cat roams left area, white text upper-right
    let cat_w: u32 = 100;
    let cat_h: u32 = 72;
    let width: u32 = 420;
    let height: u32 = 120;
    let cat_y = height - cat_h - 6;

    // Flip = face right: translate(2*x + w, 0) scale(-1, 1)
    let flip_tx = 2 * 8 + cat_w;

    // SMIL — 15s, sprite default = faces LEFT
    //
    //  0-1.5s    stand←      x=70   (native left, still)
    //  1.5-3.5s  walk←       x=70→0 (native left, moving left, 2s)
    //  3.5-5s    stand←      x=0    (native left, still)
    //  5-6s      sit         x=0    (still)
    //  6-8s      sleep       x=0    (still)
    //  8-9s      sit         x=0    (still)
    //  9-10s     petting     x=0    (still)
    // 10-11s     sit         x=0    (still)
    // 11-12s     stand→      x=0    (flipped right, still)
    // 12-14s     walk→       x=0→70 (flipped right, moving right, 2s)
    // 14-15s     stand←      x=70   (native left, still — loops)
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="{width}" height="{height}" role="img" aria-label="CommitCat badge">
  <title>CommitCat - {username}</title>

  <!-- Background -->
  <rect width="{width}" height="{height}" rx="14" fill="#7FD17F"/>

  <!-- Text (upper right, white) -->
  <g font-family="-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif" text-rendering="geometricPrecision" text-anchor="end">
    <text x="{text_x}" y="32" fill="#fff" font-size="16" font-weight="700">{username}</text>
    <text x="{text_x}" y="52" fill="rgba(255,255,255,0.85)" font-size="12">{info_line}</text>
  </g>

  <!-- Animated cat (SMIL) -->
  <g>
    <!-- Position: start at 70, walk left to 0, stay, walk right to 70 -->
    <animateTransform attributeName="transform" type="translate" dur="15s" repeatCount="indefinite"
      values="70,0; 70,0; 0,0; 0,0; 0,0; 0,0; 0,0; 0,0; 0,0; 0,0; 70,0; 70,0"
      keyTimes="0; 0.100; 0.233; 0.333; 0.400; 0.533; 0.600; 0.667; 0.733; 0.800; 0.933; 1"/>

    <!-- stand← (native): 0-1.5s, 3.5-5s, 14-15s -->
    <image x="8" y="{cat_y}" width="{cat_w}" height="{cat_h}" href="data:image/png;base64,{stand}" opacity="0">
      <animate attributeName="opacity" dur="15s" repeatCount="indefinite"
        values="1;1; 0;0; 1;1; 0;0; 1;1"
        keyTimes="0;0.099; 0.100;0.232; 0.233;0.332; 0.333;0.932; 0.933;1"/>
    </image>

    <!-- walk← (native): 1.5-3.5s -->
    <image x="8" y="{cat_y}" width="{cat_w}" height="{cat_h}" href="data:image/png;base64,{walk}" opacity="0">
      <animate attributeName="opacity" dur="15s" repeatCount="indefinite"
        values="0;0; 1;1; 0;0"
        keyTimes="0;0.099; 0.100;0.232; 0.233;1"/>
    </image>

    <!-- stand→ (flipped): 11-12s -->
    <g transform="translate({flip_tx},0) scale(-1,1)">
      <image x="8" y="{cat_y}" width="{cat_w}" height="{cat_h}" href="data:image/png;base64,{stand}" opacity="0">
        <animate attributeName="opacity" dur="15s" repeatCount="indefinite"
          values="0;0; 1;1; 0;0"
          keyTimes="0;0.732; 0.733;0.799; 0.800;1"/>
      </image>
    </g>

    <!-- walk→ (flipped): 12-14s -->
    <g transform="translate({flip_tx},0) scale(-1,1)">
      <image x="8" y="{cat_y}" width="{cat_w}" height="{cat_h}" href="data:image/png;base64,{walk}" opacity="0">
        <animate attributeName="opacity" dur="15s" repeatCount="indefinite"
          values="0;0; 1;1; 0;0"
          keyTimes="0;0.799; 0.800;0.932; 0.933;1"/>
      </image>
    </g>

    <!-- sit: 5-6s, 8-9s, 10-11s -->
    <image x="8" y="{cat_y}" width="{cat_w}" height="{cat_h}" href="data:image/png;base64,{sit}" opacity="0">
      <animate attributeName="opacity" dur="15s" repeatCount="indefinite"
        values="0;0; 1;1; 0;0; 1;1; 0;0; 1;1; 0;0"
        keyTimes="0;0.332; 0.333;0.399; 0.400;0.532; 0.533;0.599; 0.600;0.666; 0.667;0.732; 0.733;1"/>
    </image>

    <!-- sleep: 6-8s -->
    <image x="8" y="{cat_y}" width="{cat_w}" height="{cat_h}" href="data:image/png;base64,{sleep}" opacity="0">
      <animate attributeName="opacity" dur="15s" repeatCount="indefinite"
        values="0;0; 1;1; 0;0"
        keyTimes="0;0.399; 0.400;0.532; 0.533;1"/>
    </image>

    <!-- petting: 9-10s -->
    <image x="8" y="{cat_y}" width="{cat_w}" height="{cat_h}" href="data:image/png;base64,{pet}" opacity="0">
      <animate attributeName="opacity" dur="15s" repeatCount="indefinite"
        values="0;0; 1;1; 0;0"
        keyTimes="0;0.599; 0.600;0.666; 0.667;1"/>
    </image>
  </g>
</svg>"##,
        width = width,
        height = height,
        text_x = width - 16,
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
        let svg = generate_badge(295, "2026", "testuser");
        assert!(svg.contains("testuser"));
        assert!(svg.contains("295 contributions"));
        assert!(svg.contains("2026"));
    }

    #[test]
    fn badge_zero_contributions() {
        let svg = generate_badge(0, "2026", "newuser");
        assert!(svg.contains("0 contributions"));
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
