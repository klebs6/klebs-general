// ---------------- [ File: osx-wallpaper-cycler/src/set_desktop_pictures_via_system_events.rs ]
crate::ix!();

#[cfg(target_os = "macos")]
pub fn set_desktop_pictures_via_system_events(
    desktop_image_paths: &[std::path::PathBuf],
) -> Result<(), WallpaperRotatorError> {
    let script = build_system_events_set_desktop_pictures_script(desktop_image_paths);
    let _ = execute_applescript_via_nsapplescript(&script, AppleScriptActionKind::SetDesktopPictures)?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn set_desktop_pictures_via_system_events(
    _desktop_image_paths: &[std::path::PathBuf],
) -> Result<(), WallpaperRotatorError> {
    Err(WallpaperRotatorError::UnsupportedPlatform)
}
