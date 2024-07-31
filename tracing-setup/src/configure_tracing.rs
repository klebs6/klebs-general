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
