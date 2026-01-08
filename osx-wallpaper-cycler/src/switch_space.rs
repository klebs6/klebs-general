// ---------------- [ File: osx-wallpaper-cycler/src/switch_space.rs ]
crate::ix!();

pub fn build_system_events_switch_space_left_script() -> &'static str {
    r#"tell application "System Events" to key code 123 using {control down}"#
}

pub fn build_system_events_switch_space_right_script() -> &'static str {
    r#"tell application "System Events" to key code 124 using {control down}"#
}

pub fn build_system_events_set_current_desktop_picture_script(
    desktop_image_path: &std::path::Path,
) -> String {
    let escaped = escape_applescript_string(&desktop_image_path.to_string_lossy());
    format!(
        r#"
tell application "System Events"
    set picture of current desktop to POSIX file "{path}"
end tell
"#,
        path = escaped
    )
}

#[cfg(target_os = "macos")]
pub fn set_current_desktop_picture_via_system_events(
    desktop_image_path: &std::path::Path,
) -> Result<(), WallpaperRotatorError> {
    let script = build_system_events_set_current_desktop_picture_script(desktop_image_path);
    let _ = execute_applescript_via_nsapplescript(
        &script,
        AppleScriptActionKind::SetCurrentDesktopPicture,
    )?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn set_current_desktop_picture_via_system_events(
    _desktop_image_path: &std::path::Path,
) -> Result<(), WallpaperRotatorError> {
    Err(WallpaperRotatorError::UnsupportedPlatform)
}

#[cfg(target_os = "macos")]
pub fn switch_space_left_via_system_events() -> Result<(), WallpaperRotatorError> {
    let script = build_system_events_switch_space_left_script();
    let _ =
        execute_applescript_via_nsapplescript(script, AppleScriptActionKind::SwitchSpaceLeft)?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn switch_space_left_via_system_events() -> Result<(), WallpaperRotatorError> {
    Err(WallpaperRotatorError::UnsupportedPlatform)
}

#[cfg(target_os = "macos")]
pub fn switch_space_right_via_system_events() -> Result<(), WallpaperRotatorError> {
    let script = build_system_events_switch_space_right_script();
    let _ =
        execute_applescript_via_nsapplescript(script, AppleScriptActionKind::SwitchSpaceRight)?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn switch_space_right_via_system_events() -> Result<(), WallpaperRotatorError> {
    Err(WallpaperRotatorError::UnsupportedPlatform)
}

pub fn should_log_desktop_count_override_once() -> bool {
    use std::sync::atomic::{AtomicBool, Ordering};

    static DID_LOG: AtomicBool = AtomicBool::new(false);
    !DID_LOG.swap(true, Ordering::SeqCst)
}
