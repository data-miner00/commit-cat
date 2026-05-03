# Changelog

All notable changes to CommitCat are documented here.

## v0.5.0 — 2026-05-03

### Cat Profiles & Personality System
- Persistent cat profiles with create, rename, and delete support
- Active profile selection — switch between multiple cats on the fly
- Personality presets: Classic, Chill, Tsundere, Chaotic
  - Affects speech bubbles, idle behavior weights, coding chatter cadence, and AI chat tone
- Profile strip UI in settings: stable creation-order layout with active badge indicator
- Profile name debounced auto-save (350ms) with onBlur flush
- Legacy `catColor` migrated to the default profile on first upgrade
- Cross-window profile sync via `cat-profile:changed` event

---

## v0.4.0 — 2026-04-14

### Cloud / Server (v2.0–v2.1)
- Cloud API server (Rust + Axum + SQLite) deployed on Fly.io
- GitHub OAuth login + JWT-based auth
- Cross-device sync (event-based, offline-first) — `PUT /api/v1/sync`
- Animated SVG badge for GitHub README — `/badge/{username}`
  - Switched from DB-based to GitHub contributions scraping (no sync required)
  - SMIL animations for GitHub SVG compatibility (CSS `@keyframes` not allowed)
  - ETag-based caching, badge URL versioning to bypass GitHub camo cache
- Public profile page — `/profile/{username}`
  - Displays level, XP progress, commits, streaks, items, embedded badge
  - Shows leaderboard rank
  - Recent activity timeline (last 10 events)
- Leaderboard — `/leaderboard` (HTML) + `/api/v1/leaderboard` (JSON)
  - Sortable by level / commits / streak (top 50)
- Global stats API — `/api/v1/stats` (totalUsers, totalCommits, averageLevel, activeStreaks)
- Activity history API — `/api/v1/profile/{username}/activity`
- Cloud API landing page (`/`) with endpoint list and live stats widget
- Server lib/bin split for testability; 13 tests passing

### Bug fixes
- `longest_streak` no longer accidentally written from `current_streak` in sync handler
- Removed hardcoded TODO commands; replaced with real implementations

### Desktop (v1.x continued)
- VSCode Extension: session coding minutes + save count in status bar; click for stats; Reset/Open Profile commands
- Cat sprite drag/grab system polish; auto-equip items
- Item positioning per cat color (orange/brown/white)

### Infrastructure
- Dockerfile optimized: only required cat sprites copied, layer caching reordered
- `.dockerignore` added (excludes target/, node_modules/, etc.)
- Fly.io deployment with `/health` HTTP check
- Oracle Cloud Always Free deployment stack (`deploy/`): docker-compose + nginx + Let's Encrypt automation + setup guide

### Documentation
- `CONTRIBUTING.md` rewritten with full project structure and dev guide
- README: Cloud section, Built With (Axum, SQLite), roadmap updated

---

## v0.3.1 — 2026-04-01

Last release before the v0.4 cloud rollout. See [v0.3.1 release notes](https://github.com/eunseo9311/commit-cat/releases/tag/v0.3.1) for details.

---

For the full commit history, see [GitHub commits](https://github.com/eunseo9311/commit-cat/commits/main).
