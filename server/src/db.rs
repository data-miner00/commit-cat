use sqlx::SqlitePool;

pub async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:commitcat.db?mode=rwc".to_string());
    init_db_with_url(&db_url).await
}

pub async fn init_db_with_url(db_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePool::connect(db_url).await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            github_id INTEGER UNIQUE NOT NULL,
            github_username TEXT UNIQUE NOT NULL,
            github_avatar_url TEXT,
            access_token TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )"
    ).execute(&pool).await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS user_data (
            user_id TEXT PRIMARY KEY REFERENCES users(id),
            level INTEGER NOT NULL DEFAULT 1,
            exp INTEGER NOT NULL DEFAULT 0,
            total_commits INTEGER NOT NULL DEFAULT 0,
            total_coding_minutes INTEGER NOT NULL DEFAULT 0,
            current_streak INTEGER NOT NULL DEFAULT 0,
            longest_streak INTEGER NOT NULL DEFAULT 0,
            total_late_night_sessions INTEGER NOT NULL DEFAULT 0,
            current_hat TEXT,
            unlocked_hats TEXT NOT NULL DEFAULT '[]',
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )"
    ).execute(&pool).await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sync_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id TEXT NOT NULL REFERENCES users(id),
            device_id TEXT NOT NULL,
            event_type TEXT NOT NULL,
            xp INTEGER NOT NULL DEFAULT 0,
            occurred_at TEXT NOT NULL,
            synced_at TEXT NOT NULL DEFAULT (datetime('now'))
        )"
    ).execute(&pool).await?;

    Ok(pool)
}
