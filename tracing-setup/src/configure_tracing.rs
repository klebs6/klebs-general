crate::ix!();

/// Initializes the logging subscriber.
pub fn configure_tracing() {

    static INIT: std::sync::Once = std::sync::Once::new();

    INIT.call_once(|| {
        // Fall back to INFO if RUST_LOG is unset
        let filter = tracing_subscriber::EnvFilter::from_default_env()
            .add_directive(Level::DEBUG.into());  

        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .init();
        }
    );
}

pub fn setup_buffered_tracing(tag: Option<&str>) -> Arc<BufferedSubscriberLayer<Registry>> {

    let buffered_layer = match tag { 
        Some(tag) => BufferedLayer::new(tag), 
        None      => BufferedLayer::default() 
    };

    Arc::new(BufferedSubscriberLayer {
        inner: Registry::default().with(buffered_layer.clone()),
        buffered_layer: buffered_layer.into(),
    })
}
