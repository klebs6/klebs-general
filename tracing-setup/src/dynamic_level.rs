crate::ix!();

#[test] fn test_dynamic_tracing() {

    let dynamic_layer = setup_dynamic_tracing(Level::INFO);

    // Dynamically set the logging level to DEBUG
    dynamic_layer.set_level(Level::DEBUG);

    tracing::info!("This is an info message");
    tracing::debug!("This is a debug message");
}

#[derive(Clone)]
pub struct DynamicLevelLayer {
    level: Arc<AtomicUsize>,
}

impl DynamicLevelLayer {
    pub fn new(initial_level: Level) -> Self {
        Self {
            level: Arc::new(AtomicUsize::new(LogLevel::from(initial_level) as usize)),
        }
    }

    pub fn set_level(&self, level: Level) {
        self.level.store(LogLevel::from(level) as usize, Ordering::SeqCst);
    }
}

impl<S> SubscriberLayer<S> for DynamicLevelLayer
where
    S: Subscriber,
{
    fn enabled(&self, metadata: &tracing::Metadata<'_>, _ctx: Context<'_, S>) -> bool {
        let current_level = self.level.load(Ordering::SeqCst);
        metadata.level() <= &Level::from(unsafe { std::mem::transmute::<usize, LogLevel>(current_level) })
    }
}

// Function to set up dynamic tracing
pub fn setup_dynamic_tracing(initial_level: Level) -> Box<DynamicLevelLayer> {
    let dynamic_layer = Box::new(DynamicLevelLayer::new(initial_level));

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(dynamic_layer.clone())
        .with(filter)
        .init();

    dynamic_layer
}
