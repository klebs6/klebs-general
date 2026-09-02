// ---------------- [ File: osx-wallpaper-cycler/src/wallpaper_rotator_launch_agent_plist_template.rs ]
crate::ix!();

pub fn wallpaper_rotator_launch_agent_plist_template(
    binary_path: &std::path::Path,
    config_path: &std::path::Path,
) -> String {
    let bin = binary_path.to_string_lossy().to_string();
    let cfg = config_path.to_string_lossy().to_string();
    format!(
        r#"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
  <dict>
    <key>Label</key>
    <string>com.local.dropbox-wallpaper-rotator</string>

    <key>ProgramArguments</key>
    <array>
      <string>{}</string>
      <string>daemon</string>
      <string>--config</string>
      <string>{}</string>
    </array>

    <key>RunAtLoad</key>
    <true/>

    <key>KeepAlive</key>
    <true/>

    <key>StandardOutPath</key>
    <string>/tmp/dropbox-wallpaper-rotator.out.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/dropbox-wallpaper-rotator.err.log</string>
  </dict>
</plist>
"#,
        bin, cfg
    )
}

#[cfg(test)]
mod wallpaper_rotator_launch_agent_plist_template_contract_suite {
    use super::*;

    #[traced_test]
    fn plist_template_contains_label_program_arguments_and_log_paths() {
        let bin = std::path::PathBuf::from("/usr/local/bin/dropbox-wallpaper-rotator");
        let cfg = std::path::PathBuf::from("/Users/example/cfg.toml");

        let plist = wallpaper_rotator_launch_agent_plist_template(&bin, &cfg);

        assert!(plist.contains(r#"<string>com.local.dropbox-wallpaper-rotator</string>"#));
        assert!(plist.contains(r#"<key>ProgramArguments</key>"#));

        assert!(plist.contains(r#"<string>/usr/local/bin/dropbox-wallpaper-rotator</string>"#));
        assert!(plist.contains(r#"<string>daemon</string>"#));
        assert!(plist.contains(r#"<string>--config</string>"#));
        assert!(plist.contains(r#"<string>/Users/example/cfg.toml</string>"#));

        assert!(plist.contains(r#"<key>RunAtLoad</key>"#));
        assert!(plist.contains(r#"<true/>"#));

        assert!(plist.contains(r#"<key>KeepAlive</key>"#));
        assert!(plist.contains(r#"<true/>"#));

        assert!(plist.contains(r#"<key>StandardOutPath</key>"#));
        assert!(plist.contains(r#"<string>/tmp/dropbox-wallpaper-rotator.out.log</string>"#));

        assert!(plist.contains(r#"<key>StandardErrorPath</key>"#));
        assert!(plist.contains(r#"<string>/tmp/dropbox-wallpaper-rotator.err.log</string>"#));
    }

    #[traced_test]
    fn plist_template_has_xml_preamble_and_plist_root() {
        let bin = std::path::PathBuf::from("/bin/echo");
        let cfg = std::path::PathBuf::from("/tmp/cfg.toml");

        let plist = wallpaper_rotator_launch_agent_plist_template(&bin, &cfg);

        assert!(plist.contains(r#"<?xml version="1.0" encoding="UTF-8"?>"#));
        assert!(plist.contains(r#"<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "#));
        assert!(plist.contains(r#"<plist version="1.0">"#));
        assert!(plist.contains(r#"</plist>"#));
    }
}
