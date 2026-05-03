use serde::{Deserialize, Serialize};

pub const DEFAULT_CAT_PROFILE_ID: &str = "default";
pub const DEFAULT_CAT_PROFILE_NAME: &str = "My Cat";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CatColor {
    White,
    Brown,
    Orange,
}

impl Default for CatColor {
    fn default() -> Self {
        Self::Brown
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CatPersonalityPreset {
    Classic,
    Chill,
    Tsundere,
    Chaotic,
}

impl Default for CatPersonalityPreset {
    fn default() -> Self {
        Self::Classic
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CatProfile {
    #[serde(default = "default_profile_id")]
    pub id: String,
    #[serde(default = "default_profile_name")]
    pub name: String,
    #[serde(default)]
    pub color: CatColor,
    #[serde(default)]
    pub personality: CatPersonalityPreset,
}

impl Default for CatProfile {
    fn default() -> Self {
        default_cat_profile()
    }
}

pub fn default_profile_id() -> String {
    DEFAULT_CAT_PROFILE_ID.to_string()
}

pub fn default_profile_name() -> String {
    DEFAULT_CAT_PROFILE_NAME.to_string()
}

pub fn default_cat_profile() -> CatProfile {
    CatProfile {
        id: default_profile_id(),
        name: default_profile_name(),
        color: CatColor::Brown,
        personality: CatPersonalityPreset::Classic,
    }
}

pub fn normalize_cat_profiles(profiles: &mut Vec<CatProfile>, active_profile_id: &mut String) {
    if profiles.is_empty() {
        profiles.push(default_cat_profile());
    }

    for profile in profiles.iter_mut() {
        if profile.id.trim().is_empty() {
            profile.id = default_profile_id();
        }
        profile.name = normalize_profile_name(&profile.name);
    }

    let mut deduped = Vec::with_capacity(profiles.len());
    let mut seen_ids = std::collections::HashSet::new();
    for profile in profiles.drain(..) {
        if seen_ids.insert(profile.id.clone()) {
            deduped.push(profile);
        }
    }
    *profiles = deduped;

    if profiles.is_empty() {
        profiles.push(default_cat_profile());
    }

    if active_profile_id.trim().is_empty()
        || !profiles
            .iter()
            .any(|profile| profile.id == *active_profile_id)
    {
        *active_profile_id = profiles[0].id.clone();
    }
}

pub fn normalize_profile_name(name: &str) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        default_profile_name()
    } else {
        trimmed.to_string()
    }
}
