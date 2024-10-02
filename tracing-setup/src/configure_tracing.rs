crate::ix!();

/// Initializes the logging subscriber.
pub fn configure_tracing() {
    static INIT: std::sync::Once = std::sync::Once::new();

    INIT.call_once(|| {
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info"));

        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .init();
    });
}
