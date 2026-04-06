use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::cat_profile::{default_cat_profile, normalize_cat_profiles, CatColor, CatProfile};

/// 앱 설정 (권한 토글 포함)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    /// 풀스크린 자동 숨김
    pub auto_hide_fullscreen: bool,
    /// 키보드/마우스 활동 감지 허용
    pub activity_tracking: bool,
    /// IDE 감지 허용
    pub ide_detection: bool,
    /// Git 연동 허용
    pub git_integration: bool,
    /// Docker 감지 허용 (v2)
    pub docker_integration: bool,
    /// AI 기능 (v3)
    pub ai_enabled: bool,
    /// 뽀모도로 기본 시간 (분)
    pub pomodoro_minutes: u32,
    /// 휴식 시간 (분)
    #[serde(default = "default_break_minutes")]
    pub break_minutes: u32,
    /// 유휴 판정 시간 (초)
    pub idle_threshold_seconds: u64,
    /// 밤 시간 시작 (시, 24h)
    pub night_hour_start: u32,
    /// 밤 시간 끝 (시, 24h)
    pub night_hour_end: u32,
    /// 등록된 Git 저장소 경로들
    pub git_repos: Vec<String>,
    /// 알림 활성화
    #[serde(default = "default_true")]
    pub notifications_enabled: bool,
    /// GitHub PAT
    #[serde(default)]
    pub github_token: Option<String>,
    /// GitHub username (토큰 검증 시 저장)
    #[serde(default)]
    pub github_username: Option<String>,
    /// Anthropic API key (AI 채팅)
    #[serde(default)]
    pub anthropic_api_key: Option<String>,
    /// OpenAI API key (AI 채팅)
    #[serde(default)]
    pub openai_api_key: Option<String>,
    /// AI provider identifier (default: "claude")
    /// Supported values:
    /// - "claude"
    /// - "openai-api" (legacy alias: "openai")
    /// - "openai-codex-local"
    #[serde(default = "default_ai_provider")]
    pub ai_provider: String,
    /// Provider별 마지막으로 선택한 model identifier
    /// key: provider id, value: model id
    #[serde(default)]
    pub ai_provider_models: HashMap<String, String>,
    /// Provider/model별 마지막으로 선택한 reasoning effort override
    /// key: "{provider}::{model}", value: reasoning effort
    #[serde(default)]
    pub ai_provider_reasoning: HashMap<String, String>,
    /// 동료 고양이 최대 수 (0 = 메인만, 1 = +1, 2 = +2)
    #[serde(default = "default_max_companions")]
    pub max_companions: u32,
    /// deprecated: 이전 버전 호환용
    #[serde(default)]
    pub sub_cats_enabled: Option<bool>,
    /// 생일 월 (1-12)
    #[serde(default)]
    pub birthday_month: Option<u32>,
    /// 생일 일 (1-31)
    #[serde(default)]
    pub birthday_day: Option<u32>,
    /// deprecated: 프로필 도입 전 저장되었을 수 있는 레거시 고양이 색상
    #[serde(default, rename = "catColor", skip_serializing)]
    pub legacy_cat_color: Option<CatColor>,
    /// 클라우드 싱크 JWT 토큰
    #[serde(default)]
    pub cloud_token: Option<String>,
    /// 클라우드 서버 URL (기본: https://api.commitcat.dev)
    #[serde(default)]
    pub cloud_server_url: Option<String>,
    /// 디바이스 식별자
    #[serde(default)]
    pub device_id: Option<String>,
}

fn default_max_companions() -> u32 {
    2
}
fn default_break_minutes() -> u32 {
    5
}
fn default_true() -> bool {
    true
}
fn default_ai_provider() -> String {
    "claude".to_string()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            auto_hide_fullscreen: true,
            activity_tracking: true,
            ide_detection: true,
            git_integration: true,
            docker_integration: false,
            ai_enabled: false,
            pomodoro_minutes: 25,
            break_minutes: 5,
            idle_threshold_seconds: 300,
            night_hour_start: 23,
            night_hour_end: 6,
            git_repos: vec![],
            notifications_enabled: true,
            github_token: None,
            github_username: None,
            anthropic_api_key: None,
            openai_api_key: None,
            ai_provider: default_ai_provider(),
            ai_provider_models: HashMap::new(),
            ai_provider_reasoning: HashMap::new(),
            max_companions: default_max_companions(),
            sub_cats_enabled: None,
            birthday_month: None,
            birthday_day: None,
            legacy_cat_color: None,
            cloud_token: None,
            cloud_server_url: None,
            device_id: None,
        }
    }
}

/// GitHub 폴링 상태 (중복 방지)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GitHubState {
    /// repo별 알려진 PR ID 목록
    #[serde(default)]
    pub last_pr_ids: HashMap<String, Vec<u64>>,
    /// repo별 PR 상태 (open/merged/closed)
    #[serde(default)]
    pub last_pr_states: HashMap<String, HashMap<u64, String>>,
    /// repo별 마지막 star 수
    #[serde(default)]
    pub last_star_counts: HashMap<String, u32>,
}

/// 로컬 저장 전체 데이터
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppData {
    pub version: u32,
    pub settings: AppSettings,
    pub cat: CatPersistence,
    #[serde(default = "default_cat_profiles")]
    pub cat_profiles: Vec<CatProfile>,
    #[serde(default = "default_active_cat_profile_id")]
    pub active_cat_profile_id: String,
    pub today: super::activity::DailySummary,
    pub history: Vec<super::activity::DailySummary>,
    #[serde(default)]
    pub github_state: GitHubState,
    #[serde(default)]
    pub last_update_check: Option<String>,
}

/// 고양이 영구 데이터 (레벨/경험치)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatPersistence {
    pub level: u32,
    pub exp: u32,
    pub total_coding_minutes: u32,
    pub total_commits: u32,
    pub streak_days: u32,
    pub last_active_date: Option<String>,
    /// 현재 장착 모자
    #[serde(default)]
    pub current_hat: Option<String>,
    /// 해금된 모자 목록
    #[serde(default)]
    pub unlocked_hats: Vec<String>,
    /// 심야 코딩 세션 횟수 (누적)
    #[serde(default)]
    pub total_late_night_sessions: u32,
    /// 유저가 직접 선택한 모자 (자동 장착 복원용)
    #[serde(default)]
    pub preferred_hat: Option<String>,
    /// 자동 장착 만료 시각 (ISO format)
    #[serde(default)]
    pub auto_equip_until: Option<String>,
}

impl Default for CatPersistence {
    fn default() -> Self {
        Self {
            level: 1,
            exp: 0,
            total_coding_minutes: 0,
            total_commits: 0,
            streak_days: 0,
            last_active_date: None,
            current_hat: None,
            unlocked_hats: Vec::new(),
            total_late_night_sessions: 0,
            preferred_hat: None,
            auto_equip_until: None,
        }
    }
}

impl Default for AppData {
    fn default() -> Self {
        Self {
            version: 1,
            settings: AppSettings::default(),
            cat: CatPersistence::default(),
            cat_profiles: default_cat_profiles(),
            active_cat_profile_id: default_active_cat_profile_id(),
            today: super::activity::DailySummary::default(),
            history: vec![],
            github_state: GitHubState::default(),
            last_update_check: None,
        }
    }
}

fn default_cat_profiles() -> Vec<CatProfile> {
    vec![default_cat_profile()]
}

fn default_active_cat_profile_id() -> String {
    default_cat_profile().id
}

impl AppData {
    pub fn normalize(&mut self) {
        normalize_cat_profiles(&mut self.cat_profiles, &mut self.active_cat_profile_id);

        if let Some(legacy_color) = self.settings.legacy_cat_color.take() {
            let default_profile = default_cat_profile();
            if self.cat_profiles.len() == 1 {
                let profile = &mut self.cat_profiles[0];
                let is_default_profile = profile.id == default_profile.id
                    && profile.name == default_profile.name
                    && profile.color == default_profile.color
                    && profile.personality == default_profile.personality
                    && self.active_cat_profile_id == profile.id;
                if is_default_profile {
                    profile.color = legacy_color;
                }
            }
        }
    }

    pub fn active_cat_profile(&self) -> Option<&CatProfile> {
        self.cat_profiles
            .iter()
            .find(|profile| profile.id == self.active_cat_profile_id)
            .or_else(|| self.cat_profiles.first())
    }

    pub fn active_cat_profile_mut(&mut self) -> Option<&mut CatProfile> {
        if let Some(index) = self
            .cat_profiles
            .iter()
            .position(|profile| profile.id == self.active_cat_profile_id)
        {
            self.cat_profiles.get_mut(index)
        } else {
            self.cat_profiles.first_mut()
        }
    }
}
