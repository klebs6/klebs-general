// ---------------- [ File: osx-wallpaper-cycler/src/query_desktop_count_via_system_events.rs ]
crate::ix!();

#[cfg(target_os = "macos")]
pub fn query_desktop_count_via_system_events() -> Result<usize, WallpaperRotatorError> {
    let script = build_system_events_count_desktops_script();
    let desc = execute_applescript_via_nsapplescript(script, AppleScriptActionKind::CountDesktops)?;
    let count_i32: i32 = unsafe { objc2::msg_send![&*desc, int32Value] };
    if count_i32 < 0 {
        return Ok(0);
    }
    Ok(count_i32 as usize)
}

#[cfg(not(target_os = "macos"))]
pub fn query_desktop_count_via_system_events() -> Result<usize, WallpaperRotatorError> {
    Err(WallpaperRotatorError::UnsupportedPlatform)
}
