// ---------------- [ File: osx-wallpaper-cycler/src/wallpaper_rotator_invalid_configuration_details.rs ]
crate::ix!();

#[derive(Debug)]
pub struct WallpaperRotatorInvalidConfigurationDetails {
    missing_dropbox_app_key: bool,
    missing_dropbox_refresh_token: bool,
    empty_roots: bool,
    empty_allowed_extensions: bool,
    zero_interval_seconds: bool,
    zero_concurrency: bool,
}

impl WallpaperRotatorInvalidConfigurationDetails {
    pub fn new(
        missing_dropbox_app_key: bool,
        missing_dropbox_refresh_token: bool,
        empty_roots: bool,
        empty_allowed_extensions: bool,
        zero_interval_seconds: bool,
        zero_concurrency: bool,
    ) -> Self {
        Self {
            missing_dropbox_app_key,
            missing_dropbox_refresh_token,
            empty_roots,
            empty_allowed_extensions,
            zero_interval_seconds,
            zero_concurrency,
        }
    }

    pub fn any_invalid(&self) -> bool {
        self.missing_dropbox_app_key
            || self.missing_dropbox_refresh_token
            || self.empty_roots
            || self.empty_allowed_extensions
            || self.zero_interval_seconds
            || self.zero_concurrency
    }
}

impl std::fmt::Display for WallpaperRotatorInvalidConfigurationDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut problems: Vec<&'static str> = Vec::new();

        if self.missing_dropbox_app_key {
            problems.push("dropbox_app_key missing");
        }
        if self.missing_dropbox_refresh_token {
            problems.push("dropbox_refresh_token missing");
        }
        if self.empty_roots {
            problems.push("dropbox_roots empty");
        }
        if self.empty_allowed_extensions {
            problems.push("allowed_extensions empty");
        }
        if self.zero_interval_seconds {
            problems.push("interval_seconds must be > 0");
        }
        if self.zero_concurrency {
            problems.push("concurrency must be > 0");
        }

        write!(f, "{}", problems.join(", "))
    }
}

#[cfg(test)]
mod wallpaper_rotator_invalid_configuration_details_contract_suite {
    use super::*;

    #[traced_test]
    fn any_invalid_is_false_when_all_flags_are_false() {
        let d = WallpaperRotatorInvalidConfigurationDetails::new(false, false, false, false, false, false);
        assert!(!d.any_invalid());
    }

    #[traced_test]
    fn any_invalid_is_true_when_any_flag_is_true() {
        let d = WallpaperRotatorInvalidConfigurationDetails::new(true, false, false, false, false, false);
        assert!(d.any_invalid());

        let d = WallpaperRotatorInvalidConfigurationDetails::new(false, true, false, false, false, false);
        assert!(d.any_invalid());

        let d = WallpaperRotatorInvalidConfigurationDetails::new(false, false, true, false, false, false);
        assert!(d.any_invalid());

        let d = WallpaperRotatorInvalidConfigurationDetails::new(false, false, false, true, false, false);
        assert!(d.any_invalid());

        let d = WallpaperRotatorInvalidConfigurationDetails::new(false, false, false, false, true, false);
        assert!(d.any_invalid());

        let d = WallpaperRotatorInvalidConfigurationDetails::new(false, false, false, false, false, true);
        assert!(d.any_invalid());
    }

    #[traced_test]
    fn display_lists_all_problems_in_stable_order_when_all_invalid() {
        let d = WallpaperRotatorInvalidConfigurationDetails::new(true, true, true, true, true, true);
        let s = d.to_string();

        assert_eq!(
            s,
            "dropbox_app_key missing, dropbox_refresh_token missing, dropbox_roots empty, allowed_extensions empty, interval_seconds must be > 0, concurrency must be > 0"
        );
    }

    #[traced_test]
    fn display_lists_single_problem_without_commas_when_one_invalid() {
        let d = WallpaperRotatorInvalidConfigurationDetails::new(false, false, false, false, true, false);
        assert_eq!(d.to_string(), "interval_seconds must be > 0");
    }
}
