// ---------------- [ File: osx-wallpaper-cycler/src/wallpaper_rotator_configuration_contract_suite.rs ]
crate::ix!();

#[cfg(test)]
mod wallpaper_rotator_configuration_contract_suite {
    use super::*;

    #[traced_test]
    fn extension_normalization_strips_dot_and_normalizes_case() {
        assert_eq!(normalize_extension_token("JPG"), "jpg");
        assert_eq!(normalize_extension_token(".PNG"), "png");
        assert_eq!(normalize_extension_token("  .HeIc  "), "heic");
        assert_eq!(normalize_extension_token(""), "");
        assert_eq!(normalize_extension_token("   "), "");
    }

    #[traced_test]
    fn config_validation_rejects_missing_required_fields_and_invalid_numbers() {
        let cfg = WallpaperRotatorConfigBuilder::default()
            .dropbox_app_key("")
            .dropbox_refresh_token("")
            .dropbox_roots(Vec::<String>::new())
            .allowed_extensions(Vec::<String>::new())
            .interval_seconds(0u64)
            .concurrency(0usize)
            .scope(WallpaperRotatorScope::AllDesktops)
            .build()
            .unwrap();

        let err = cfg.validate_or_error().err().unwrap();
        match err {
            WallpaperRotatorError::InvalidConfiguration { .. } => {}
            _ => panic!("unexpected error variant"),
        }
    }

    #[traced_test]
    fn config_validation_accepts_minimal_valid_config() {
        let cfg = WallpaperRotatorConfigBuilder::default()
            .dropbox_app_key("k")
            .dropbox_refresh_token("r")
            .dropbox_roots(vec!["/Wallpapers".to_string()])
            .allowed_extensions(vec!["jpg".to_string()])
            .interval_seconds(1u64)
            .concurrency(1usize)
            .scope(WallpaperRotatorScope::AllDesktops)
            .build()
            .unwrap();

        assert!(cfg.validate_or_error().is_ok());
        let set = cfg.normalized_allowed_extensions_set();
        assert!(set.contains("jpg"));
    }

    #[traced_test]
    fn tilde_expansion_expands_when_home_available() {
        use std::sync::{Mutex, OnceLock};

        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let lock = ENV_LOCK.get_or_init(|| Mutex::new(()));

        let _guard = lock.lock().unwrap();

        let old = std::env::var_os("HOME");
        unsafe { std::env::set_var("HOME", "/tmp") };

        let p = expand_tilde_path(std::path::Path::new("~/x/y"));
        assert_eq!(p, std::path::PathBuf::from("/tmp").join("x").join("y"));

        unsafe {
            if let Some(old) = old {
                std::env::set_var("HOME", old);
            } else {
                std::env::remove_var("HOME");
            }
        }
    }

    #[traced_test]
    fn effective_cache_dir_prefers_override_and_expands_tilde() {
        use std::sync::{Mutex, OnceLock};

        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let lock = ENV_LOCK.get_or_init(|| Mutex::new(()));

        let _guard = lock.lock().unwrap();

        let old = std::env::var_os("HOME");
        unsafe { std::env::set_var("HOME", "/tmp") };

        let cfg = WallpaperRotatorConfigBuilder::default()
            .dropbox_app_key("k")
            .dropbox_refresh_token("r")
            .dropbox_roots(vec!["/Wallpapers".to_string()])
            .allowed_extensions(vec!["jpg".to_string()])
            .interval_seconds(1u64)
            .cache_dir(std::path::PathBuf::from("~/cache_override"))
            .concurrency(1usize)
            .scope(WallpaperRotatorScope::AllDesktops)
            .build()
            .unwrap();

        let dir = cfg.effective_cache_dir().unwrap();
        assert_eq!(dir, std::path::PathBuf::from("/tmp").join("cache_override"));

        unsafe {
            if let Some(old) = old {
                std::env::set_var("HOME", old);
            } else {
                std::env::remove_var("HOME");
            }
        }
    }

    #[traced_test]
    fn toml_load_round_trips_for_valid_minimal_config() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let dir = tempfile::TempDir::new().unwrap();
            let path = dir.path().join("cfg.toml");
            let content = r#"
dropbox_app_key = "k"
dropbox_refresh_token = "r"
dropbox_roots = ["/Wallpapers"]
allowed_extensions = ["jpg"]
interval_seconds = 1
concurrency = 1
scope = "all_desktops"
"#;
            tokio::fs::write(&path, content).await.unwrap();
            let cfg = load_wallpaper_rotator_config_from_toml(&path).await.unwrap();
            assert_eq!(cfg.dropbox_app_key(), "k");
            assert_eq!(cfg.dropbox_refresh_token(), "r");
            assert_eq!(cfg.interval_seconds(), &1u64);
            assert_eq!(cfg.concurrency(), &1usize);
            assert_eq!(*cfg.scope(), WallpaperRotatorScope::AllDesktops);
        });
    }

    #[traced_test]
    fn toml_load_rejects_invalid_config() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let dir = tempfile::TempDir::new().unwrap();
            let path = dir.path().join("cfg.toml");
            let content = r#"
dropbox_app_key = ""
dropbox_refresh_token = ""
dropbox_roots = []
allowed_extensions = []
interval_seconds = 0
concurrency = 0
scope = "all_desktops"
"#;
            tokio::fs::write(&path, content).await.unwrap();
            let err = load_wallpaper_rotator_config_from_toml(&path).await.err().unwrap();
            match err {
                WallpaperRotatorError::InvalidConfiguration { .. } => {}
                _ => panic!("unexpected error variant"),
            }
        });
    }
}
