// ---------------- [ File: osx-wallpaper-cycler/src/build_system_events_set_desktop_pictures_script.rs ]
crate::ix!();

pub fn build_system_events_set_desktop_pictures_script(
    desktop_image_paths: &[std::path::PathBuf],
) -> String {
    let mut apple_list: String = String::new();
    apple_list.push('{');

    for (idx, p) in desktop_image_paths.iter().enumerate() {
        if idx > 0 {
            apple_list.push_str(", ");
        }
        let escaped = escape_applescript_string(&p.to_string_lossy());
        apple_list.push('"');
        apple_list.push_str(&escaped);
        apple_list.push('"');
    }

    apple_list.push('}');

    format!(
        r#"
tell application "System Events"
    set imagePaths to {image_list}
    if (count of imagePaths) is 0 then
        error number -1
    end if

    set deskGroup to (a reference to every desktop)
    set desktopCount to count of deskGroup

    repeat with i from 1 to desktopCount
        set idx to ((i - 1) mod (count of imagePaths)) + 1
        set p to item idx of imagePaths
        set theDesk to item i of deskGroup
        set picture of theDesk to (POSIX file p as alias)
    end repeat
end tell
"#,
        image_list = apple_list
    )
}
