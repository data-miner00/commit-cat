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
        format!(" | {}d streak", streak)
    } else {
        String::new()
    };
    let info_line = format!("Lv.{}  {}c{}", level, total_commits, streak_text);

    // Layout: [cat roaming area 200px] [circle badge on right]
    let circle_r: u32 = 42;
    let width: u32 = 330;
    let height: u32 = 120;
    let cx = width - circle_r - 18; // circle center x
    let cy = height / 2;            // circle center y
    let cat_w: u32 = 100;           // cat sprite width
    let cat_h: u32 = 72;            // cat sprite height
    let cat_y = height - cat_h - 8; // cat y position (bottom-aligned)

    // SMIL animation — proper state machine (16s total):
    //
    //   stand→ → walk→ → stand→ → sit → sleep → sit → petting → sit → stand← → walk← → stand→
    //   (→ = facing right, ← = facing left)
    //
    //    0-1s   stand→      x=0
    //    1-3.5s walk→       x=0 → x=80
    //    3.5-4.5s stand→    x=80
    //    4.5-5.5s sit       x=80
    //    5.5-7.5s sleep     x=80
    //    7.5-8.5s sit       x=80
    //    8.5-9.5s petting   x=80
    //    9.5-10.5s sit      x=80
    //   10.5-11s stand←     x=80
    //   11-13.5s walk←      x=80 → x=0
    //   13.5-14s stand→     x=0  (seamless loop back)
    //   But to be clean: 14s → resting at start = 16s total
    //
    // keyTimes (16s):
    //  1/16=0.0625   3.5/16=0.219   4.5/16=0.281   5.5/16=0.344
    //  7.5/16=0.469  8.5/16=0.531   9.5/16=0.594  10.5/16=0.656
    // 11/16=0.688   13.5/16=0.844  14.5/16=0.906  16/16=1.0
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="{width}" height="{height}" role="img" aria-label="CommitCat badge">
  <title>CommitCat - {username}</title>

  <!-- Background -->
  <rect width="{width}" height="{height}" rx="14" fill="#7FD17F"/>

  <!-- Circle badge -->
  <circle cx="{cx}" cy="{cy}" r="{circle_r}" fill="#fff" opacity="0.92"/>
  <g font-family="-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif" text-rendering="geometricPrecision" text-anchor="middle">
    <text x="{cx}" y="{uname_y}" fill="#2a6e2a" font-size="13" font-weight="700">{username}</text>
    <text x="{cx}" y="{info_y}" fill="#666" font-size="10">{info_line}</text>
  </g>

  <!-- Cat sprites (facing right) -->
  <g>
    <animateTransform attributeName="transform" type="translate" dur="16s" repeatCount="indefinite"
      values="0,0; 0,0; 80,0; 80,0; 80,0; 80,0; 80,0; 80,0; 80,0; 80,0; 80,0; 0,0; 0,0; 0,0"
      keyTimes="0; 0.0625; 0.219; 0.281; 0.344; 0.469; 0.531; 0.594; 0.656; 0.688; 0.844; 0.845; 0.906; 1"
      calcMode="linear"/>

    <!-- stand→: 0-1s, 3.5-4.5s, 13.5-16s (facing right) -->
    <image x="8" y="{cat_y}" width="{cat_w}" height="{cat_h}" href="data:image/png;base64,{stand}" opacity="0">
      <animate attributeName="opacity" dur="16s" repeatCount="indefinite"
        values="1;1; 0;0; 1;1; 0;0; 0;0; 0;0; 0;0; 0;0; 0;0; 1;1"
        keyTimes="0;0.062; 0.063;0.218; 0.219;0.280; 0.281;0.343; 0.344;0.468; 0.469;0.530; 0.531;0.593; 0.594;0.655; 0.656;0.905; 0.906;1"/>
    </image>

    <!-- walk→: 1-3.5s (facing right) -->
    <image x="8" y="{cat_y}" width="{cat_w}" height="{cat_h}" href="data:image/png;base64,{walk}" opacity="0">
      <animate attributeName="opacity" dur="16s" repeatCount="indefinite"
        values="0;0; 1;1; 0;0"
        keyTimes="0;0.062; 0.063;0.218; 0.219;1"/>
    </image>

    <!-- stand←: 10.5-11s (facing left, flipped) -->
    <g transform="translate({flip_stand},0) scale(-1,1)">
      <image x="8" y="{cat_y}" width="{cat_w}" height="{cat_h}" href="data:image/png;base64,{stand}" opacity="0">
        <animate attributeName="opacity" dur="16s" repeatCount="indefinite"
          values="0;0; 1;1; 0;0"
          keyTimes="0;0.655; 0.656;0.687; 0.688;1"/>
      </image>
    </g>

    <!-- walk←: 11-13.5s (facing left, flipped) -->
    <g transform="translate({flip_walk},0) scale(-1,1)">
      <image x="8" y="{cat_y}" width="{cat_w}" height="{cat_h}" href="data:image/png;base64,{walk}" opacity="0">
        <animate attributeName="opacity" dur="16s" repeatCount="indefinite"
          values="0;0; 1;1; 0;0"
          keyTimes="0;0.687; 0.688;0.844; 0.845;1"/>
      </image>
    </g>

    <!-- sit: 4.5-5.5s, 7.5-8.5s, 9.5-10.5s -->
    <image x="8" y="{cat_y}" width="{cat_w}" height="{cat_h}" href="data:image/png;base64,{sit}" opacity="0">
      <animate attributeName="opacity" dur="16s" repeatCount="indefinite"
        values="0;0; 1;1; 0;0; 1;1; 0;0; 1;1; 0;0"
        keyTimes="0;0.280; 0.281;0.343; 0.344;0.468; 0.469;0.530; 0.531;0.593; 0.594;0.655; 0.656;1"/>
    </image>

    <!-- sleep: 5.5-7.5s -->
    <image x="8" y="{cat_y}" width="{cat_w}" height="{cat_h}" href="data:image/png;base64,{sleep}" opacity="0">
      <animate attributeName="opacity" dur="16s" repeatCount="indefinite"
        values="0;0; 1;1; 0;0"
        keyTimes="0;0.343; 0.344;0.468; 0.469;1"/>
    </image>

    <!-- petting: 8.5-9.5s -->
    <image x="8" y="{cat_y}" width="{cat_w}" height="{cat_h}" href="data:image/png;base64,{pet}" opacity="0">
      <animate attributeName="opacity" dur="16s" repeatCount="indefinite"
        values="0;0; 1;1; 0;0"
        keyTimes="0;0.530; 0.531;0.593; 0.594;1"/>
    </image>
  </g>
</svg>"##,
        width = width,
        height = height,
        cx = cx,
        cy = cy,
        circle_r = circle_r,
        uname_y = cy - 4,
        info_y = cy + 12,
        cat_y = cat_y,
        cat_w = cat_w,
        cat_h = cat_h,
        flip_stand = 2 * 8 + cat_w, // translate for horizontal flip
        flip_walk = 2 * 8 + cat_w,
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
