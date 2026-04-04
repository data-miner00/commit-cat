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
    let stand_b64 = BASE64.encode(CAT_STAND);
    let walk_b64 = BASE64.encode(CAT_WALK);
    let sit_b64 = BASE64.encode(CAT_SIT);
    let sleep_b64 = BASE64.encode(CAT_SLEEP);
    let pet_b64 = BASE64.encode(CAT_PETTING);

    let info_line = format!("{} contributions in {}", contributions, year);

    let cat_w: u32 = 100;
    let cat_h: u32 = 72;
    let width: u32 = 420;
    let height: u32 = 120;
    let cat_y = height - cat_h - 6;
    let flip_tx = 2 * 8 + cat_w;

    // Sprite faces LEFT natively. Flip for RIGHT.
    // Phase-based animation: define sequence, auto-compute keyTimes.
    //
    // Sprite indices:
    //   0 = stand native (left)
    //   1 = walk native (left)
    //   2 = stand flipped (right)
    //   3 = walk flipped (right)
    //   4 = sit
    //   5 = sleep
    //   6 = petting
    //
    // (sprite, x_start, x_end, duration_secs)
    let phases: &[(u8, u32, u32, f64)] = &[
        // Start at right side, look left
        (0, 190, 190, 1.5),   // stand← at 190
        (1, 190, 100, 1.0),   // walk← to 100
        (0, 100, 100, 1.5),   // stand← at 100
        (4, 100, 100, 1.5),   // sit
        (5, 100, 100, 3.0),   // sleep
        (4, 100, 100, 1.0),   // sit (wake up)
        // Turn right, walk to right
        (2, 100, 100, 0.8),   // stand→
        (3, 100, 200, 1.0),   // walk→ to 200
        (2, 200, 200, 1.5),   // stand→ at 200
        (4, 200, 200, 1.5),   // sit
        (6, 200, 200, 2.0),   // petting
        (4, 200, 200, 1.0),   // sit
        // Turn left, walk to left side
        (0, 200, 200, 0.8),   // stand←
        (1, 200, 60, 1.0),    // walk← to 60
        (0, 60, 60, 1.5),     // stand← at 60
        (4, 60, 60, 2.0),     // sit
        (5, 60, 60, 2.5),     // sleep
        (4, 60, 60, 1.0),     // sit (wake up)
        (6, 60, 60, 1.5),     // petting
        (4, 60, 60, 0.8),     // sit
        // Turn right, walk back to start position
        (2, 60, 60, 0.8),     // stand→
        (3, 60, 140, 1.0),    // walk→ to 140
        (2, 140, 140, 1.0),   // stand→ briefly
        (3, 140, 190, 1.0),   // walk→ to 190 (back to loop start)
    ];

    let total: f64 = phases.iter().map(|p| p.3).sum();
    let dur = total.ceil() as u32;

    // Build translate keyTimes/values
    let mut translate_times = Vec::new();
    let mut translate_values = Vec::new();
    let mut t = 0.0_f64;
    for phase in phases {
        let t_start = t / total;
        translate_times.push(format!("{:.4}", t_start));
        translate_values.push(format!("{},0", phase.1));
        t += phase.3;
        let t_end = t / total;
        translate_times.push(format!("{:.4}", t_end.min(1.0)));
        translate_values.push(format!("{},0", phase.2));
    }
    // Deduplicate consecutive identical time entries
    let mut final_times = vec![translate_times[0].clone()];
    let mut final_values = vec![translate_values[0].clone()];
    for i in 1..translate_times.len() {
        if translate_times[i] != *final_times.last().unwrap() {
            final_times.push(translate_times[i].clone());
            final_values.push(translate_values[i].clone());
        }
    }
    // Ensure ends at 1.0000
    if let Some(last) = final_times.last_mut() {
        *last = "1.0000".to_string();
    }
    let translate_kt = final_times.join("; ");
    let translate_v = final_values.join("; ");

    // Build opacity keyTimes/values for each sprite.
    // For each sprite: walk through ALL phases, output 0 or 1 at each boundary.
    // 7 sprites: stand←(0), walk←(1), stand→(2), walk→(3), sit(4), sleep(5), petting(6)
    let eps = 0.001;
    let mut sprite_opacities: Vec<(Vec<String>, Vec<String>)> = Vec::new();
    for s in 0..7u8 {
        let mut kt = Vec::new();
        let mut vl = Vec::new();
        let mut t_acc = 0.0_f64;
        let first_sprite = phases[0].0;

        // Initial state
        kt.push("0.0000".to_string());
        vl.push(if s == first_sprite { "1" } else { "0" }.to_string());

        for phase in phases {
            let t_start = t_acc / total;
            let _t_end = (t_acc + phase.3) / total;
            let is_me = phase.0 == s;
            let was_on = vl.last().map(|v| v.as_str()) == Some("1");

            if is_me && !was_on {
                // Turn on: insert 0 just before, then 1
                let t_before = (t_start - eps).max(0.0);
                if t_before > 0.0 {
                    kt.push(format!("{:.4}", t_before));
                    vl.push("0".to_string());
                }
                kt.push(format!("{:.4}", t_start));
                vl.push("1".to_string());
            } else if !is_me && was_on {
                // Turn off: insert 1 just before, then 0
                let t_before = (t_start - eps).max(0.0);
                kt.push(format!("{:.4}", t_before));
                vl.push("1".to_string());
                kt.push(format!("{:.4}", t_start));
                vl.push("0".to_string());
            }
            t_acc += phase.3;
        }

        // Close: match first frame for seamless loop
        let last_val = vl.last().map(|v| v.as_str()).unwrap_or("0");
        let loop_val = if s == first_sprite { "1" } else { "0" };
        if last_val != loop_val {
            kt.push(format!("{:.4}", (1.0 - eps)));
            vl.push(last_val.to_string());
            kt.push("1.0000".to_string());
            vl.push(loop_val.to_string());
        } else {
            kt.push("1.0000".to_string());
            vl.push(loop_val.to_string());
        }

        sprite_opacities.push((kt, vl));
    }

    let opacity_attr = |idx: usize| -> (String, String) {
        let (ref kt, ref vl) = sprite_opacities[idx];
        (kt.join("; "), vl.join("; "))
    };

    let (kt0, v0) = opacity_attr(0); // stand←
    let (kt1, v1) = opacity_attr(1); // walk←
    let (kt2, v2) = opacity_attr(2); // stand→
    let (kt3, v3) = opacity_attr(3); // walk→
    let (kt4, v4) = opacity_attr(4); // sit
    let (kt5, v5) = opacity_attr(5); // sleep
    let (kt6, v6) = opacity_attr(6); // petting

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="{width}" height="{height}" role="img" aria-label="CommitCat badge">
  <title>CommitCat - {username}</title>
  <rect width="{width}" height="{height}" rx="14" fill="#7FD17F"/>
  <g font-family="-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif" text-rendering="geometricPrecision" text-anchor="end">
    <text x="{text_x}" y="32" fill="#fff" font-size="16" font-weight="700">{username}</text>
    <text x="{text_x}" y="52" fill="rgba(255,255,255,0.85)" font-size="12">{info_line}</text>
  </g>
  <g>
    <animateTransform attributeName="transform" type="translate" dur="{dur}s" repeatCount="indefinite" values="{translate_v}" keyTimes="{translate_kt}"/>
    <image x="8" y="{cat_y}" width="{cat_w}" height="{cat_h}" href="data:image/png;base64,{stand}" opacity="0">
      <animate attributeName="opacity" dur="{dur}s" repeatCount="indefinite" values="{v0}" keyTimes="{kt0}"/>
    </image>
    <image x="8" y="{cat_y}" width="{cat_w}" height="{cat_h}" href="data:image/png;base64,{walk}" opacity="0">
      <animate attributeName="opacity" dur="{dur}s" repeatCount="indefinite" values="{v1}" keyTimes="{kt1}"/>
    </image>
    <g transform="translate({flip_tx},0) scale(-1,1)">
      <image x="8" y="{cat_y}" width="{cat_w}" height="{cat_h}" href="data:image/png;base64,{stand}" opacity="0">
        <animate attributeName="opacity" dur="{dur}s" repeatCount="indefinite" values="{v2}" keyTimes="{kt2}"/>
      </image>
    </g>
    <g transform="translate({flip_tx},0) scale(-1,1)">
      <image x="8" y="{cat_y}" width="{cat_w}" height="{cat_h}" href="data:image/png;base64,{walk}" opacity="0">
        <animate attributeName="opacity" dur="{dur}s" repeatCount="indefinite" values="{v3}" keyTimes="{kt3}"/>
      </image>
    </g>
    <image x="8" y="{cat_y}" width="{cat_w}" height="{cat_h}" href="data:image/png;base64,{sit}" opacity="0">
      <animate attributeName="opacity" dur="{dur}s" repeatCount="indefinite" values="{v4}" keyTimes="{kt4}"/>
    </image>
    <image x="8" y="{cat_y}" width="{cat_w}" height="{cat_h}" href="data:image/png;base64,{sleep}" opacity="0">
      <animate attributeName="opacity" dur="{dur}s" repeatCount="indefinite" values="{v5}" keyTimes="{kt5}"/>
    </image>
    <image x="8" y="{cat_y}" width="{cat_w}" height="{cat_h}" href="data:image/png;base64,{pet}" opacity="0">
      <animate attributeName="opacity" dur="{dur}s" repeatCount="indefinite" values="{v6}" keyTimes="{kt6}"/>
    </image>
  </g>
</svg>"##,
        width = width,
        height = height,
        text_x = width - 16,
        cat_y = cat_y,
        cat_w = cat_w,
        cat_h = cat_h,
        dur = dur,
        flip_tx = flip_tx,
        username = username,
        info_line = info_line,
        translate_v = translate_v,
        translate_kt = translate_kt,
        stand = stand_b64,
        walk = walk_b64,
        sit = sit_b64,
        sleep = sleep_b64,
        pet = pet_b64,
        v0 = v0, kt0 = kt0,
        v1 = v1, kt1 = kt1,
        v2 = v2, kt2 = kt2,
        v3 = v3, kt3 = kt3,
        v4 = v4, kt4 = kt4,
        v5 = v5, kt5 = kt5,
        v6 = v6, kt6 = kt6,
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
