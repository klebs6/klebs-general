// ---------------- [ File: osx-wallpaper-cycler/src/wallpaper_rotator_config.rs ]
crate::ix!();

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, getset::Getters, derive_builder::Builder)]
#[getset(get = "pub")]
#[builder(setter(into))]
pub struct WallpaperRotatorConfig {
    dropbox_app_key: String,

    #[builder(default)]
    dropbox_app_secret: Option<String>,

    dropbox_refresh_token: String,
    dropbox_roots: Vec<String>,
    allowed_extensions: Vec<String>,
    interval_seconds: u64,

    #[builder(default)]
    cache_dir: Option<std::path::PathBuf>,

    concurrency: usize,
    scope: WallpaperRotatorScope,

    #[builder(default)]
    #[serde(default)]
    force_dropbox_directory_cache_rescan: bool,
}

impl WallpaperRotatorConfig {
    pub fn validate_or_error(&self) -> Result<(), WallpaperRotatorError> {
        let missing_dropbox_app_key = self.dropbox_app_key.trim().is_empty();
        let missing_dropbox_refresh_token = self.dropbox_refresh_token.trim().is_empty();
        let empty_roots = self.dropbox_roots.is_empty();
        let empty_allowed_extensions = self.allowed_extensions.is_empty();
        let zero_interval_seconds = self.interval_seconds == 0;
        let zero_concurrency = self.concurrency == 0;

        let details = WallpaperRotatorInvalidConfigurationDetails::new(
            missing_dropbox_app_key,
            missing_dropbox_refresh_token,
            empty_roots,
            empty_allowed_extensions,
            zero_interval_seconds,
            zero_concurrency,
        );

        if details.any_invalid() {
            return Err(wallpaper_rotator_invalid_configuration(details));
        }

        Ok(())
    }

    pub fn normalized_allowed_extensions_set(&self) -> std::collections::HashSet<String> {
        let mut set = std::collections::HashSet::new();
        for ext in self.allowed_extensions.iter() {
            let normalized = normalize_extension_token(ext);
            if !normalized.is_empty() {
                set.insert(normalized);
            }
        }
        set
    }

    pub fn effective_cache_dir(&self) -> Result<std::path::PathBuf, WallpaperRotatorError> {
        if let Some(path) = self.cache_dir.as_ref() {
            return Ok(expand_tilde_path(path));
        }

        let home = std::env::var_os("HOME")
            .map(std::path::PathBuf::from)
            .ok_or(WallpaperRotatorError::HomeDirectoryUnavailable)?;

        Ok(home
            .join("Library")
            .join("Caches")
            .join("dropbox_wallpaper_rotator"))
    }
}
