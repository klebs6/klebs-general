crate::ix!();

pub fn setup_dynamic_tracing(initial_level: Level) 
    -> reload::Handle<EnvFilter, impl tracing::Subscriber + Send + Sync> 
{
    // Create an EnvFilter with the initial level
    let filter = EnvFilter::from_default_env()
        .add_directive(initial_level.into());

    // Create a reloadable layer
    let (filter_layer, reload_handle) = reload::Layer::new(filter);

    // Build the subscriber
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(initial_level)
        .finish()
        .with(filter_layer);

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    reload_handle
}

#[cfg(test)]
mod dynamic_tracing_tests {
    use super::*;
    use tracing::Level;
    use tracing_subscriber::EnvFilter;

    #[test]
    fn test_dynamic_tracing() {
        // Set up dynamic tracing with initial level INFO
        let reload_handle = setup_dynamic_tracing(Level::INFO);

        // Log messages at different levels
        tracing::info!("This is an info message");
        tracing::debug!("This debug message should NOT appear at INFO level");

        // Dynamically set the logging level to DEBUG
        reload_handle.reload(EnvFilter::new("debug")).unwrap();

        tracing::debug!("This debug message should appear at DEBUG level");
    }
}
