// ---------------- [ File: osx-wallpaper-cycler/src/system_events_applescript_script_generation_suite.rs ]
crate::ix!();

#[cfg(test)]
mod system_events_applescript_script_generation_suite {
    use super::*;

    #[traced_test]
    fn applescript_escape_handles_quotes_backslashes_and_controls() {
        assert_eq!(escape_applescript_string(r#"a"b"#), r#"a\"b"#);
        assert_eq!(escape_applescript_string(r#"a\b"#), r#"a\\b"#);
        assert_eq!(escape_applescript_string("a\nb"), r#"a\nb"#);
        assert_eq!(escape_applescript_string("a\rb"), r#"a\rb"#);
        assert_eq!(escape_applescript_string("a\tb"), r#"a\tb"#);
    }

    #[traced_test]
    fn build_set_desktop_pictures_script_contains_modulo_logic_and_paths() {
        let paths = vec![
            std::path::PathBuf::from("/tmp/a.jpg"),
            std::path::PathBuf::from("/tmp/b.png"),
        ];
        let script = build_system_events_set_desktop_pictures_script(&paths);

        assert!(script.contains(r#"tell application "System Events""#));
        assert!(script.contains(r#"set idx to ((i - 1) mod (count of imagePaths)) + 1"#));
        assert!(script.contains(r#""/tmp/a.jpg""#));
        assert!(script.contains(r#""/tmp/b.png""#));

        assert!(script.contains("every desktop"));
        assert!(script.contains("set picture of"));
    }

    #[traced_test]
    fn build_set_current_desktop_picture_script_mentions_current_desktop_and_path() {
        let p = std::path::PathBuf::from("/tmp/current.png");
        let script = build_system_events_set_current_desktop_picture_script(p.as_path());
        assert!(script.contains(r#"tell application "System Events""#));
        assert!(script.contains(r#"picture of current desktop"#));
        assert!(script.contains(r#""/tmp/current.png""#));
    }

    #[traced_test]
    fn build_switch_space_scripts_use_expected_key_codes() {
        let left = build_system_events_switch_space_left_script();
        let right = build_system_events_switch_space_right_script();

        assert!(left.contains("key code 123"));
        assert!(left.contains("{control down}"));

        assert!(right.contains("key code 124"));
        assert!(right.contains("{control down}"));
    }

    #[traced_test]
    fn count_desktops_script_is_expected() {
        assert_eq!(
            build_system_events_count_desktops_script(),
            r#"tell application "System Events" to count desktops"#
        );
    }
}
