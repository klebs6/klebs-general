// ---------------- [ File: osx-wallpaper-cycler/src/execute_applescript_via_nsapplescript.rs ]
crate::ix!();

#[cfg(target_os = "macos")]
pub fn execute_applescript_via_nsapplescript(
    source: &str,
    action: AppleScriptActionKind,
) -> Result<objc2::rc::Retained<objc2_foundation::NSAppleEventDescriptor>, WallpaperRotatorError> {
    use objc2::rc::Retained;
    use objc2::runtime::AnyObject;
    use objc2::AnyThread;
    use objc2_foundation::{NSDictionary, NSAppleEventDescriptor, NSAppleScript, NSString};

    tracing::trace!(
        action = %action,
        source_bytes = source.len(),
        "compiling and executing AppleScript"
    );

    let ns_source: Retained<NSString> = NSString::from_str(source);

    let script: Option<Retained<NSAppleScript>> =
        unsafe { objc2::msg_send![NSAppleScript::alloc(), initWithSource: &*ns_source] };

    let Some(script) = script else {
        tracing::error!(
            action = %action,
            source_bytes = source.len(),
            "NSAppleScript initWithSource returned nil"
        );
        return Err(wallpaper_rotator_applescript_failure(action, None));
    };

    let mut error_dict: Option<Retained<NSDictionary<NSString, AnyObject>>> = None;

    let result: Option<Retained<NSAppleEventDescriptor>> =
        unsafe { objc2::msg_send![&*script, executeAndReturnError: &mut error_dict] };

    if let Some(desc) = result {
        tracing::trace!(action = %action, "AppleScript execution succeeded");
        return Ok(desc);
    }

    let error_number = error_dict
        .as_ref()
        .and_then(|d| try_extract_applescript_error_number(d));

    let error_app_name = error_dict.as_ref().and_then(|d| {
        try_extract_applescript_error_string(d, "NSAppleScriptErrorAppName")
    });

    let error_brief_message = error_dict.as_ref().and_then(|d| {
        try_extract_applescript_error_string(d, "NSAppleScriptErrorBriefMessage")
    });

    let error_message = error_dict.as_ref().and_then(|d| {
        try_extract_applescript_error_string(d, "NSAppleScriptErrorMessage")
    });

    let error_range = error_dict.as_ref().and_then(|d| {
        try_extract_applescript_error_string(d, "NSAppleScriptErrorRange")
    });

    tracing::warn!(
        action = %action,
        error_number,
        error_dict_present = error_dict.is_some(),
        error_app_name = ?error_app_name,
        error_brief_message = ?error_brief_message,
        error_message = ?error_message,
        error_range = ?error_range,
        "AppleScript execution returned nil"
    );

    if error_number == Some(1002) {
        match std::env::current_exe() {
            Ok(exe) => {
                tracing::error!(
                    action = %action,
                    exe = %exe.display(),
                    "UI scripting is not permitted for this executable; enable Accessibility permission (System Settings → Privacy & Security → Accessibility) to allow keystroke-based Space switching"
                );
            }
            Err(e) => {
                tracing::error!(
                    action = %action,
                    error = %e,
                    "UI scripting is not permitted for this executable; enable Accessibility permission (System Settings → Privacy & Security → Accessibility) to allow keystroke-based Space switching"
                );
            }
        }
    }

    Err(wallpaper_rotator_applescript_failure(action, error_number))
}
