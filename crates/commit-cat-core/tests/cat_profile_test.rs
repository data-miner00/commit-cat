use commit_cat_core::models::cat_profile::{
    normalize_cat_profiles, CatColor, CatPersonalityPreset, CatProfile, DEFAULT_CAT_PROFILE_ID,
};
use commit_cat_core::models::settings::AppData;

#[test]
fn normalize_profiles_creates_default_when_missing() {
    let mut profiles = Vec::new();
    let mut active_profile_id = String::new();

    normalize_cat_profiles(&mut profiles, &mut active_profile_id);

    assert_eq!(profiles.len(), 1);
    assert_eq!(profiles[0].id, DEFAULT_CAT_PROFILE_ID);
    assert_eq!(profiles[0].name, "My Cat");
    assert_eq!(profiles[0].color, CatColor::Brown);
    assert_eq!(profiles[0].personality, CatPersonalityPreset::Classic);
    assert_eq!(active_profile_id, DEFAULT_CAT_PROFILE_ID);
}

#[test]
fn normalize_profiles_repairs_invalid_active_id() {
    let mut data = AppData::default();
    data.cat_profiles = vec![
        CatProfile {
            id: "alpha".to_string(),
            name: "Alpha".to_string(),
            color: CatColor::White,
            personality: CatPersonalityPreset::Chill,
        },
        CatProfile {
            id: "beta".to_string(),
            name: "Beta".to_string(),
            color: CatColor::Orange,
            personality: CatPersonalityPreset::Chaotic,
        },
    ];
    data.active_cat_profile_id = "missing".to_string();

    data.normalize();

    assert_eq!(data.active_cat_profile_id, "alpha");
}
