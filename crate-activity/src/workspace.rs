crate::ix!();

pub async fn configure_directory() -> Result<PathBuf, CrateActivityError> {
    let config_dir = dirs::home_dir()
        .map(|p| p.join(".published-crates"))
        .unwrap_or_else(|| PathBuf::from(".published-crates"));
    ensure_config_structure(&config_dir).await?;
    Ok(config_dir)
}
