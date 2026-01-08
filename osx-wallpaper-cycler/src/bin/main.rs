// ---------------- [ File: osx-wallpaper-cycler/src/bin/main.rs ]
use osx_wallpaper_cycler::*;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    initialize_tracing_subscriber_with_env_filter();

    let args = <WallpaperRotatorCli as structopt::StructOpt>::from_args();

    match args.command() {
        WallpaperRotatorCliCommand::Daemon { config } => {
            match load_wallpaper_rotator_config_from_toml(config).await {
                Ok(cfg) => {
                    if let Err(e) = run_wallpaper_rotation_daemon_loop(cfg).await {
                        tracing::error!(error = %e, "daemon terminated with error");
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, path = %config.display(), "failed to load config");
                }
            }
        }
        WallpaperRotatorCliCommand::RunOnce { config } => {
            match load_wallpaper_rotator_config_from_toml(config).await {
                Ok(cfg) => {
                    let source: std::sync::Arc<dyn DropboxWallpaperSource> =
                        match DropboxApiClient::new(cfg.dropbox_app_key().clone(), cfg.dropbox_app_secret().clone(), cfg.dropbox_refresh_token().clone()) {
                            Ok(c) => std::sync::Arc::new(c),
                            Err(e) => {
                                tracing::error!(error = %e, "failed to initialize dropbox client");
                                return;
                            }
                        };

                    let controller: std::sync::Arc<dyn DesktopWallpaperController> =
                        std::sync::Arc::new(SystemEventsDesktopWallpaperController::new());

                    if let Err(e) = perform_wallpaper_rotation_cycle_once(&cfg, source, controller).await {
                        tracing::error!(error = %e, "run-once failed");
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, path = %config.display(), "failed to load config");
                }
            }
        }
        WallpaperRotatorCliCommand::PrintConfigTemplate => {
            print!("{}", wallpaper_rotator_config_toml_template());
        }
        WallpaperRotatorCliCommand::PrintLaunchAgentTemplate {
            binary_path,
            config_path,
        } => {
            let plist = wallpaper_rotator_launch_agent_plist_template(binary_path, config_path);
            print!("{plist}");
        }
    }
}
