// ---------------- [ File: osx-wallpaper-cycler/src/run_wallpaper_rotation_daemon_loop.rs ]
crate::ix!();

pub async fn run_wallpaper_rotation_daemon_loop(
    cfg: WallpaperRotatorConfig,
) -> Result<(), WallpaperRotatorError> {
    cfg.validate_or_error()?;

    let source: std::sync::Arc<dyn DropboxWallpaperSource> = std::sync::Arc::new(DropboxApiClient::new(
        cfg.dropbox_app_key().clone(),
        cfg.dropbox_app_secret().clone(),
        cfg.dropbox_refresh_token().clone(),
    )?);

    let controller: std::sync::Arc<dyn DesktopWallpaperController> =
        std::sync::Arc::new(SystemEventsDesktopWallpaperController::new());

    let interval = std::time::Duration::from_secs(*cfg.interval_seconds());
    let mut ticker = tokio::time::interval(interval);

    tracing::info!(
        interval_seconds = cfg.interval_seconds(),
        scope = %cfg.scope(),
        concurrency = cfg.concurrency(),
        "starting daemon loop"
    );

    loop {
        ticker.tick().await;

        let span = tracing::info_span!("wallpaper_rotation_tick");
        let _enter = span.enter();

        match perform_wallpaper_rotation_cycle_once(&cfg, source.clone(), controller.clone()).await {
            Ok(_) => tracing::info!("tick completed successfully"),
            Err(e) => tracing::error!(error = %e, "tick failed"),
        }
    }
}
