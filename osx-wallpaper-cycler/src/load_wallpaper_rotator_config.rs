// ---------------- [ File: osx-wallpaper-cycler/src/load_wallpaper_rotator_config.rs ]
crate::ix!();

pub async fn load_wallpaper_rotator_config_from_toml(
    config_path: &std::path::Path,
) -> Result<WallpaperRotatorConfig, WallpaperRotatorError> {
    let content = tokio::fs::read_to_string(config_path).await.map_err(|e| {
        wallpaper_rotator_io_failure(FilesystemOperationKind::ReadFile, config_path.to_path_buf(), e)
    })?;

    let cfg: WallpaperRotatorConfig =
        toml::from_str(&content).map_err(|e| WallpaperRotatorError::TomlDecodeFailure {
            path: config_path.to_path_buf(),
            source: e,
        })?;

    cfg.validate_or_error()?;
    Ok(cfg)
}
