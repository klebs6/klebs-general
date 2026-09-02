// ---------------- [ File: osx-wallpaper-cycler/src/applescript.rs ]
crate::ix!();

pub fn build_system_events_count_desktops_script() -> &'static str {
    r#"tell application "System Events" to count desktops"#
}
