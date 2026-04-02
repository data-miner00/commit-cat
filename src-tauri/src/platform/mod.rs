#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub mod linux;

use commit_cat_core::models::settings::AppData;
use commit_cat_core::platform::{EventEmitter, Storage};
use serde_json::Value;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};

/// OS별 IDE 감지 (통합 인터페이스)
pub fn detect_running_ide() -> Option<String> {
    #[cfg(target_os = "macos")]
    { macos::detect_running_ide() }

    #[cfg(target_os = "windows")]
    { windows::detect_running_ide() }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    { linux::detect_running_ide() }
}

/// OS별 IDE PID 목록 (통합 인터페이스)
pub fn get_ide_pids() -> Vec<u32> {
    #[cfg(target_os = "macos")]
    { macos::get_ide_pids() }

    #[cfg(target_os = "windows")]
    { windows::get_ide_pids() }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    { linux::get_ide_pids() }
}

/// OS별 프로세스 cwd 추출 (통합 인터페이스)
pub fn get_process_cwd(pid: u32) -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    { macos::get_process_cwd(pid) }

    #[cfg(target_os = "windows")]
    { windows::get_process_cwd(pid) }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    { linux::get_process_cwd(pid) }
}

/// Tauri event emitter implementation
pub struct TauriEmitter {
    pub app: AppHandle,
}

impl EventEmitter for TauriEmitter {
    fn emit(&self, event: &str, payload: Value) {
        self.app.emit(event, payload).ok();
    }
}

/// Tauri storage implementation (delegates to existing storage service)
pub struct TauriStorage {
    pub app: AppHandle,
}

impl Storage for TauriStorage {
    fn load(&self) -> Result<AppData, Box<dyn std::error::Error>> {
        crate::services::storage::load(&self.app).map_err(|e| -> Box<dyn std::error::Error> { e.into() })
    }

    fn save(&self, data: &AppData) -> Result<(), Box<dyn std::error::Error>> {
        crate::services::storage::save(&self.app, data).map_err(|e| -> Box<dyn std::error::Error> { e.into() })
    }
}
