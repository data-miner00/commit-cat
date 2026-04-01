use crate::services;
use tauri::AppHandle;

#[tauri::command]
pub async fn clone_repo(app: AppHandle, url: String, path: String) -> Result<bool, String> {
    let clone_path = path.clone();
    tokio::task::spawn_blocking(move || {
        git2::Repository::clone(&url, &clone_path).map_err(|e| format!("Clone failed: {}", e))
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))??;
    services::storage::add_repo(&app, &path)?;
    Ok(true)
}

#[tauri::command]
pub async fn get_today_commits(app: tauri::AppHandle) -> Result<u32, String> {
    let repos = services::storage::get_watched_repos(&app);
    let total: u32 = repos
        .iter()
        .map(|r| services::git::count_today_commits(r))
        .sum();
    Ok(total)
}

#[tauri::command]
pub async fn register_repo(app: tauri::AppHandle, path: String) -> Result<bool, String> {
    let git_dir = std::path::Path::new(&path).join(".git");
    if git_dir.exists() {
        services::storage::add_repo(&app, &path)?;
        Ok(true)
    } else {
        Err("Not a valid git repository".to_string())
    }
}

#[tauri::command]
pub async fn get_watched_repos(app: tauri::AppHandle) -> Result<Vec<String>, String> {
    let repos = services::storage::get_watched_repos(&app);
    Ok(repos
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect())
}

#[tauri::command]
pub async fn remove_repo(app: tauri::AppHandle, path: String) -> Result<bool, String> {
    services::storage::remove_repo(&app, &path)?;
    Ok(true)
}
