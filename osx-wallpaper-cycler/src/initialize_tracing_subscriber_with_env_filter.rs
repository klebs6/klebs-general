// ---------------- [ File: osx-wallpaper-cycler/src/initialize_tracing_subscriber_with_env_filter.rs ]
crate::ix!();

pub(crate) fn derive_tracing_level_from_rust_log_filter_string(env_filter: &str) -> tracing::Level {
    let env_filter_lc = env_filter.trim().to_ascii_lowercase();

    if env_filter_lc.contains("trace") {
        tracing::Level::TRACE
    } else if env_filter_lc.contains("debug") {
        tracing::Level::DEBUG
    } else if env_filter_lc.contains("info") {
        tracing::Level::INFO
    } else if env_filter_lc.contains("warn") {
        tracing::Level::WARN
    } else if env_filter_lc.contains("error") {
        tracing::Level::ERROR
    } else {
        tracing::Level::INFO
    }
}

pub fn initialize_tracing_subscriber_with_env_filter() {
    let env_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    let level = derive_tracing_level_from_rust_log_filter_string(&env_filter);

    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();
}

#[cfg(test)]
mod tracing_level_derivation_contract_suite {
    use super::*;

    #[traced_test]
    fn derives_most_verbose_level_based_on_substring_precedence() {
        assert_eq!(
            derive_tracing_level_from_rust_log_filter_string("trace"),
            tracing::Level::TRACE
        );
        assert_eq!(
            derive_tracing_level_from_rust_log_filter_string("DEBUG"),
            tracing::Level::DEBUG
        );
        assert_eq!(
            derive_tracing_level_from_rust_log_filter_string("info"),
            tracing::Level::INFO
        );
        assert_eq!(
            derive_tracing_level_from_rust_log_filter_string("warn"),
            tracing::Level::WARN
        );
        assert_eq!(
            derive_tracing_level_from_rust_log_filter_string("error"),
            tracing::Level::ERROR
        );

        assert_eq!(
            derive_tracing_level_from_rust_log_filter_string("something_else"),
            tracing::Level::INFO
        );

        assert_eq!(
            derive_tracing_level_from_rust_log_filter_string("error,foo=debug"),
            tracing::Level::DEBUG
        );
        assert_eq!(
            derive_tracing_level_from_rust_log_filter_string("warn,foo=trace"),
            tracing::Level::TRACE
        );
    }
}
