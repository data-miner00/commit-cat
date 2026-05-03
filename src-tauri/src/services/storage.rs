use commit_cat_core::models::settings::AppData;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

const DATA_FILE: &str = "commit-cat-data.json";
const BACKUP_FILE: &str = "commit-cat-data.backup.json";

/// 파일 쓰기 동시 접근 방지
static SAVE_LOCK: Mutex<()> = Mutex::new(());

/// 앱 데이터 디렉토리 경로
fn data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    Ok(dir)
}

/// 초기화: 데이터 파일이 없으면 기본값으로 생성
pub fn init(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let dir = data_dir(app)?;
    std::fs::create_dir_all(&dir)?;

    let path = dir.join(DATA_FILE);
    if !path.exists() {
        let default_data = AppData::default();
        let json = serde_json::to_string_pretty(&default_data)?;
        std::fs::write(&path, json)?;
    }
    Ok(())
}

/// 데이터 로드 (파싱 실패 시 백업에서 복구)
pub fn load(app: &AppHandle) -> Result<AppData, String> {
    let mut data = load_from_dir(&data_dir(app)?)?;
    data.normalize();
    Ok(data)
}

/// 앱 시작 시 1회 호출 — 날짜가 바뀌었으면 DailySummary 리셋
pub fn check_daily_reset(app: &AppHandle) -> Result<(), String> {
    let _lock = SAVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = data_dir(app)?;
    let mut data = load_from_dir(&dir)?;
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    if data.today.date != today {
        data.today = commit_cat_core::models::activity::DailySummary {
            date: today,
            ..Default::default()
        };
        save_to_dir(&dir, &data)?;
    }
    Ok(())
}

/// 데이터 저장 (atomic write + 백업)
pub fn save(app: &AppHandle, data: &AppData) -> Result<(), String> {
    let _lock = SAVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    save_to_dir(&data_dir(app)?, data)
}

/// 데이터를 읽고 수정한 뒤 같은 락 범위에서 저장
pub fn update<R, F>(app: &AppHandle, mutate: F) -> Result<R, String>
where
    F: FnOnce(&mut AppData) -> Result<R, String>,
{
    let _lock = SAVE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = data_dir(app)?;
    let mut data = load_from_dir(&dir)?;
    let result = mutate(&mut data)?;
    save_to_dir(&dir, &data)?;
    Ok(result)
}

fn load_from_dir(dir: &Path) -> Result<AppData, String> {
    let path = dir.join(DATA_FILE);
    let content =
        std::fs::read_to_string(&path).map_err(|e| format!("Failed to read data: {}", e))?;

    match serde_json::from_str::<AppData>(&content) {
        Ok(mut data) => {
            data.normalize();
            Ok(data)
        }
        Err(e) => {
            let backup_path = dir.join(BACKUP_FILE);
            if backup_path.exists() {
                let backup_content = std::fs::read_to_string(&backup_path)
                    .map_err(|e| format!("Failed to read backup: {}", e))?;
                let mut data: AppData = serde_json::from_str(&backup_content)
                    .map_err(|_| format!("Both data and backup corrupted: {}", e))?;
                data.normalize();
                let _ = std::fs::copy(&backup_path, &path);
                Ok(data)
            } else {
                Err(format!("Failed to parse data: {}", e))
            }
        }
    }
}

fn save_to_dir(dir: &Path, data: &AppData) -> Result<(), String> {
    let path = dir.join(DATA_FILE);
    let backup_path = dir.join(BACKUP_FILE);
    let tmp_path = dir.join("commit-cat-data.tmp.json");

    let mut normalized = data.clone();
    normalized.normalize();
    let json = serde_json::to_string_pretty(&normalized)
        .map_err(|e| format!("Failed to serialize: {}", e))?;

    // 1. 현재 파일을 백업
    if path.exists() {
        let _ = std::fs::copy(&path, &backup_path);
    }

    // 2. 임시 파일에 쓰기
    std::fs::write(&tmp_path, &json).map_err(|e| format!("Failed to write temp: {}", e))?;

    // 3. 임시 파일 → 메인 파일로 rename (atomic)
    std::fs::rename(&tmp_path, &path).map_err(|e| format!("Failed to rename: {}", e))
}

/// History 관리: 90일 초과 데이터 정리
#[allow(dead_code)]
pub fn cleanup_history(data: &mut AppData) {
    if data.history.len() > 90 {
        data.history.truncate(90);
    }
}

/// 등록된 Git 저장소 목록 반환
pub fn get_watched_repos(app: &AppHandle) -> Vec<PathBuf> {
    load(app)
        .map(|data| data.settings.git_repos.iter().map(PathBuf::from).collect())
        .unwrap_or_default()
}

/// Git 저장소 삭제
pub fn remove_repo(app: &AppHandle, path: &str) -> Result<(), String> {
    let mut data = load(app)?;
    data.settings.git_repos.retain(|r| r != path);
    save(app, &data)?;
    Ok(())
}

/// Git 저장소 등록 (중복 무시)
pub fn add_repo(app: &AppHandle, path: &str) -> Result<(), String> {
    let mut data = load(app)?;
    let canonical = std::fs::canonicalize(path)
        .map_err(|e| format!("Invalid path: {}", e))?
        .to_string_lossy()
        .to_string();

    if !data.settings.git_repos.contains(&canonical) {
        data.settings.git_repos.push(canonical);
        save(app, &data)?;
    }
    Ok(())
}
