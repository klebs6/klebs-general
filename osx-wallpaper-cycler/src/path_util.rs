// ---------------- [ File: osx-wallpaper-cycler/src/path_util.rs ]
crate::ix!();

pub fn normalize_extension_token(token: &str) -> String {
    let trimmed = token.trim();
    let no_dot = trimmed.strip_prefix('.').unwrap_or(trimmed);
    no_dot.to_ascii_lowercase()
}

pub fn expand_tilde_path(path: &std::path::Path) -> std::path::PathBuf {
    let s = path.to_string_lossy();
    if let Some(rest) = s.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return std::path::PathBuf::from(home).join(rest);
        }
    }
    path.to_path_buf()
}
