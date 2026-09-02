crate::ix!();

#[cfg(not(target_os = "macos"))]
pub fn set_desktop_pictures_across_spaces_via_system_events_by_switching(
    _desktop_image_paths: &[std::path::PathBuf],
) -> Result<(), WallpaperRotatorError> {
    Err(WallpaperRotatorError::UnsupportedPlatform)
}

#[cfg(target_os = "macos")]
pub fn set_desktop_pictures_across_spaces_via_system_events_by_switching(
    desktop_image_paths: &[std::path::PathBuf],
) -> Result<(), WallpaperRotatorError> {
    const PREP_LEFT_SWIPES: usize = 32;
    const PREP_LEFT_DELAY: std::time::Duration = std::time::Duration::from_millis(60);
    const BETWEEN_SET_AND_SWITCH_DELAY: std::time::Duration = std::time::Duration::from_millis(120);
    const SWITCH_RIGHT_DELAY: std::time::Duration = std::time::Duration::from_millis(240);

    if desktop_image_paths.is_empty() {
        tracing::warn!(
            action = %AppleScriptActionKind::SetCurrentDesktopPicture,
            "requested to set wallpapers across spaces but no image paths were provided"
        );
        return Err(wallpaper_rotator_applescript_failure(
            AppleScriptActionKind::SetCurrentDesktopPicture,
            Some(-1),
        ));
    }

    let accessibility_trusted = is_process_trusted_for_accessibility();
    if !accessibility_trusted {
        match std::env::current_exe() {
            Ok(exe) => {
                tracing::error!(
                    action = %AppleScriptActionKind::SwitchSpaceLeft,
                    accessibility_trusted,
                    exe = %exe.display(),
                    "Space switching requires permission to send keystrokes; enable Accessibility permission (System Settings → Privacy & Security → Accessibility) for this executable"
                );
            }
            Err(e) => {
                tracing::error!(
                    action = %AppleScriptActionKind::SwitchSpaceLeft,
                    accessibility_trusted,
                    error = %e,
                    "Space switching requires permission to send keystrokes; enable Accessibility permission (System Settings → Privacy & Security → Accessibility) for this executable"
                );
            }
        }

        return Err(wallpaper_rotator_applescript_failure(
            AppleScriptActionKind::SwitchSpaceLeft,
            Some(1002),
        ));
    }

    tracing::info!(
        targets = desktop_image_paths.len(),
        prep_left_swipes = PREP_LEFT_SWIPES,
        prep_left_delay_ms = PREP_LEFT_DELAY.as_millis() as u64,
        between_set_and_switch_delay_ms = BETWEEN_SET_AND_SWITCH_DELAY.as_millis() as u64,
        switch_right_delay_ms = SWITCH_RIGHT_DELAY.as_millis() as u64,
        "applying wallpapers across Spaces by switching and setting current desktop"
    );

    for i in 0..PREP_LEFT_SWIPES {
        tracing::trace!(
            swipe_index = i + 1,
            swipes_total = PREP_LEFT_SWIPES,
            "preparing by switching to the left-most Space"
        );

        if let Err(e) = switch_space_left_via_system_events() {
            tracing::error!(
                swipe_index = i + 1,
                swipes_total = PREP_LEFT_SWIPES,
                error = %e,
                "failed while preparing to reach the left-most Space"
            );
            return Err(e);
        }

        std::thread::sleep(PREP_LEFT_DELAY);
    }

    for (idx, p) in desktop_image_paths.iter().enumerate() {
        tracing::debug!(
            target_index = idx + 1,
            target_total = desktop_image_paths.len(),
            path = %p.display(),
            "setting wallpaper for current Space"
        );

        if let Err(e) = set_current_desktop_picture_via_system_events(p.as_path()) {
            tracing::error!(
                target_index = idx + 1,
                target_total = desktop_image_paths.len(),
                path = %p.display(),
                error = %e,
                "failed to set current Space wallpaper"
            );
            return Err(e);
        }

        std::thread::sleep(BETWEEN_SET_AND_SWITCH_DELAY);

        if idx + 1 < desktop_image_paths.len() {
            tracing::trace!(
                from_target_index = idx + 1,
                from_target_total = desktop_image_paths.len(),
                "switching to next Space"
            );

            if let Err(e) = switch_space_right_via_system_events() {
                tracing::error!(
                    from_target_index = idx + 1,
                    from_target_total = desktop_image_paths.len(),
                    error = %e,
                    "failed to switch to the next Space"
                );
                return Err(e);
            }

            std::thread::sleep(SWITCH_RIGHT_DELAY);
        }
    }

    tracing::info!(
        targets = desktop_image_paths.len(),
        "completed Space-switch wallpaper application"
    );

    Ok(())
}
