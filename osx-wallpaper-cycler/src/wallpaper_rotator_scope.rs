// ---------------- [ File: osx-wallpaper-cycler/src/wallpaper_rotator_scope.rs ]
crate::ix!();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WallpaperRotatorScope {
    AllDesktops,
    ActiveDesktopOnly,
}

impl std::fmt::Display for WallpaperRotatorScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AllDesktops => write!(f, "all_desktops"),
            Self::ActiveDesktopOnly => write!(f, "active_desktop_only"),
        }
    }
}

#[cfg(test)]
mod wallpaper_rotator_scope_contract_suite {
    use super::*;

    #[traced_test]
    fn display_matches_serde_snake_case_and_is_unique() {
        let a = WallpaperRotatorScope::AllDesktops;
        let b = WallpaperRotatorScope::ActiveDesktopOnly;

        assert_eq!(a.to_string(), "all_desktops");
        assert_eq!(b.to_string(), "active_desktop_only");
        assert_ne!(a.to_string(), b.to_string());
    }

    #[traced_test]
    fn serde_serialization_round_trips_for_all_variants() {
        let a = WallpaperRotatorScope::AllDesktops;
        let b = WallpaperRotatorScope::ActiveDesktopOnly;

        let aj = serde_json::to_string(&a).unwrap();
        let bj = serde_json::to_string(&b).unwrap();

        assert_eq!(aj, r#""all_desktops""#);
        assert_eq!(bj, r#""active_desktop_only""#);

        let a2: WallpaperRotatorScope = serde_json::from_str(&aj).unwrap();
        let b2: WallpaperRotatorScope = serde_json::from_str(&bj).unwrap();

        assert_eq!(a2, a);
        assert_eq!(b2, b);
    }
}
