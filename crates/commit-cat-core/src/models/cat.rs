use serde::{Deserialize, Serialize};

/// 고양이의 현재 상태 (상태 머신)
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum CatState {
    #[default]
    Idle,
    Coding,
    Celebrating,
    Frustrated,
    Sleeping,
    Tired,
    Interaction,
}

/// 고양이의 현재 표정/이모지
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CatMood {
    Happy,      // 😺
    Sad,        // 😿
    Sleeping,   // 😴
    Focused,    // 🔥
    Excited,    // 💥
}

impl From<&CatState> for CatMood {
    fn from(state: &CatState) -> Self {
        match state {
            CatState::Idle => CatMood::Happy,
            CatState::Coding => CatMood::Focused,
            CatState::Celebrating => CatMood::Excited,
            CatState::Frustrated => CatMood::Sad,
            CatState::Sleeping => CatMood::Sleeping,
            CatState::Tired => CatMood::Sad,
            CatState::Interaction => CatMood::Happy,
        }
    }
}

/// 프론트엔드로 전달하는 고양이 전체 상태
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatInfo {
    pub state: CatState,
    pub mood: CatMood,
    pub level: u32,
    pub exp: u32,
    pub exp_to_next: u32,
    pub streak_days: u32,
}
