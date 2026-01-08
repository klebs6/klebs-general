// ---------------- [ File: osx-wallpaper-cycler/src/wallpaper_rotator_config_toml.rs ]
crate::ix!();

pub fn wallpaper_rotator_config_toml_template() -> &'static str {
    r#"
dropbox_app_key = "YOUR_APP_KEY"
# Optional (PKCE apps may not need a secret):
# dropbox_app_secret = "YOUR_APP_SECRET"
dropbox_refresh_token = "YOUR_REFRESH_TOKEN"

# One or more Dropbox paths to scan (recursive):
dropbox_roots = ["/Wallpapers", "/MoreWallpapers"]

# File extensions to consider (case-insensitive, dot optional):
allowed_extensions = ["jpg", "jpeg", "png", "heic", "tiff"]

# How often to rotate (daemon mode) in seconds:
interval_seconds = 1800

# Optional cache dir (defaults to ~/Library/Caches/dropbox_wallpaper_rotator):
# cache_dir = "/Users/you/Library/Caches/dropbox_wallpaper_rotator"

# Max simultaneous downloads:
concurrency = 4

# all_desktops: attempt to set wallpapers across all desktops (Spaces) via System Events.
# active_desktop_only: attempt to set only the active desktop.
scope = "all_desktops"

# Optional: set true to ignore the cached Dropbox directory index and force a rescan.
# (Useful if you add/remove files in Dropbox and want the next run to refresh immediately.)
force_dropbox_directory_cache_rescan = false
"#
}
