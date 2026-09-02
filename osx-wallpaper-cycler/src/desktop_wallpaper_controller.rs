// ---------------- [ File: osx-wallpaper-cycler/src/desktop_wallpaper_controller.rs ]
crate::ix!();

pub trait DesktopWallpaperController: Send + Sync {
    fn query_desktop_count(&self) -> Result<usize, WallpaperRotatorError>;
    fn apply_wallpapers_to_desktops(
        &self,
        scope: WallpaperRotatorScope,
        desktop_image_paths: &[std::path::PathBuf],
    ) -> Result<(), WallpaperRotatorError>;
}

#[derive(Debug, Clone, Copy)]
pub struct SystemEventsDesktopWallpaperController;

impl SystemEventsDesktopWallpaperController {
    pub fn new() -> Self {
        Self
    }
}

impl DesktopWallpaperController for SystemEventsDesktopWallpaperController {
    fn query_desktop_count(&self) -> Result<usize, WallpaperRotatorError> {
        if let Some(n) = try_read_desktop_count_override_from_env() {
            tracing::debug!(
                desktop_count = n,
                source = "env_override",
                "desktop count resolved"
            );
            return Ok(n);
        }

        let n = query_desktop_count_via_system_events()?;

        if n > 1 {
            tracing::debug!(
                desktop_count = n,
                source = "system_events",
                "desktop count resolved"
            );
            return Ok(n);
        }

        let inferred = try_infer_space_count_via_defaults_export_plutil();

        match inferred {
            Some(m) if m > n => {
                tracing::warn!(
                    system_events_desktop_count = n,
                    inferred_space_count = m,
                    source = "com.apple.spaces",
                    "System Events reported <= 1 desktop; using inferred Space count"
                );
                Ok(m)
            }
            Some(m) => {
                tracing::debug!(
                    system_events_desktop_count = n,
                    inferred_space_count = m,
                    source = "system_events",
                    "Space inference did not exceed System Events; using System Events desktop count"
                );
                Ok(n)
            }
            None => {
                tracing::debug!(
                    system_events_desktop_count = n,
                    source = "system_events",
                    "Space inference unavailable; using System Events desktop count"
                );
                Ok(n)
            }
        }
    }

    fn apply_wallpapers_to_desktops(
        &self,
        scope: WallpaperRotatorScope,
        desktop_image_paths: &[std::path::PathBuf],
    ) -> Result<(), WallpaperRotatorError> {
        tracing::info!(
            scope = %scope,
            images = desktop_image_paths.len(),
            env_override_present = try_read_desktop_count_override_from_env().is_some(),
            "applying wallpapers via System Events controller"
        );

        match scope {
            WallpaperRotatorScope::AllDesktops => {
                let observed_count = query_desktop_count_via_system_events().unwrap_or_else(|e| {
                    tracing::warn!(
                        error = %e,
                        "failed to query desktop count via System Events during apply; assuming 1"
                    );
                    1usize
                });

                let should_switch = desktop_image_paths.len() > 1 && observed_count <= 1;

                if try_read_desktop_count_override_from_env().is_some() {
                    tracing::warn!(
                        observed_desktop_count = observed_count,
                        targets = desktop_image_paths.len(),
                        "desktop count override is set; applying across Spaces by switching"
                    );
                    return set_desktop_pictures_across_spaces_via_system_events_by_switching(
                        desktop_image_paths,
                    );
                }

                if should_switch {
                    tracing::warn!(
                        observed_desktop_count = observed_count,
                        targets = desktop_image_paths.len(),
                        "System Events reported <=1 desktop; applying across Spaces by switching"
                    );
                    return set_desktop_pictures_across_spaces_via_system_events_by_switching(
                        desktop_image_paths,
                    );
                }

                let result = set_desktop_pictures_via_system_events(desktop_image_paths);

                if result.is_ok() {
                    if let Some(first) = desktop_image_paths.first() {
                        if let Err(e) = set_current_desktop_picture_via_system_events(first.as_path())
                        {
                            tracing::warn!(
                                error = %e,
                                path = %first.display(),
                                "set_desktop_pictures succeeded but setting current desktop picture failed"
                            );
                        } else {
                            tracing::debug!(
                                path = %first.display(),
                                "ensured current desktop picture is set after bulk application"
                            );
                        }
                    }
                }

                result
            }
            WallpaperRotatorScope::ActiveDesktopOnly => {
                let Some(first) = desktop_image_paths.first() else {
                    tracing::warn!("active_desktop_only was requested but no image paths were provided");
                    return Err(wallpaper_rotator_applescript_failure(
                        AppleScriptActionKind::SetCurrentDesktopPicture,
                        Some(-1),
                    ));
                };

                if desktop_image_paths.len() > 1 {
                    tracing::debug!(
                        images = desktop_image_paths.len(),
                        "active_desktop_only received multiple images; only the first will be applied"
                    );
                }

                set_current_desktop_picture_via_system_events(first.as_path())
            }
        }
    }
}

#[cfg(test)]
mod desktop_wallpaper_controller_smoke_contract_suite {
    use super::*;

    #[traced_test]
    fn system_events_controller_is_constructible_and_copyable() {
        let a = SystemEventsDesktopWallpaperController::new();
        let b = a;
        let _c = b;
    }

    #[traced_test]
    fn desktop_count_override_env_var_is_used_when_present_even_off_macos() {
        use std::sync::{Mutex, OnceLock};

        static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let lock = ENV_LOCK.get_or_init(|| Mutex::new(()));
        let _guard = lock.lock().unwrap();

        let key = "OSX_WALLPAPER_CYCLER_DESKTOP_COUNT_OVERRIDE";
        let old = std::env::var_os(key);

        unsafe {
            std::env::set_var(key, "7");
        }

        let c = SystemEventsDesktopWallpaperController::new();
        let n = c.query_desktop_count().unwrap();
        assert_eq!(n, 7);

        unsafe {
            if let Some(old) = old {
                std::env::set_var(key, old);
            } else {
                std::env::remove_var(key);
            }
        }
    }
}
