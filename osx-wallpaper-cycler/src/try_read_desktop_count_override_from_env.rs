crate::ix!();

pub fn try_read_desktop_count_override_from_env() -> Option<usize> {
    const VAR: &str = "OSX_WALLPAPER_CYCLER_DESKTOP_COUNT_OVERRIDE";

    let raw = std::env::var(VAR).ok()?;
    let trimmed = raw.trim();

    if trimmed.is_empty() {
        return None;
    }

    match trimmed.parse::<usize>() {
        Ok(n) if n > 0 => {
            if should_log_desktop_count_override_once() {
                tracing::info!(
                    env_var = %VAR,
                    desktop_count_override = n,
                    "using desktop count override from environment"
                );
            }
            Some(n)
        }
        Ok(_) => {
            if should_log_desktop_count_override_once() {
                tracing::warn!(
                    env_var = %VAR,
                    value = %trimmed,
                    "desktop count override is zero; ignoring"
                );
            }
            None
        }
        Err(e) => {
            if should_log_desktop_count_override_once() {
                tracing::warn!(
                    env_var = %VAR,
                    value = %trimmed,
                    error = %e,
                    "failed to parse desktop count override; ignoring"
                );
            }
            None
        }
    }
}
