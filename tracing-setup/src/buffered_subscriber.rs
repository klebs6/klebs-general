crate::ix!();

pub trait BufferedSubscriber: Subscriber + Flushable {}

#[derive(Debug)]
pub struct BufferedSubscriberLayer<S> {
    inner:          tracing_subscriber::layer::Layered<BufferedLayer, S>,
    buffered_layer: Arc<BufferedLayer>,
}

impl<S: Subscriber> Flushable for BufferedSubscriberLayer<S> {

    fn flush(&self) {
        self.buffered_layer.flush();
    }
}

impl<S: Subscriber> Subscriber for BufferedSubscriberLayer<S> {
    fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
        self.inner.enabled(metadata)
    }

    fn new_span(&self, span: &tracing::span::Attributes<'_>) -> tracing::Id {
        self.inner.new_span(span)
    }

    fn record(&self, span: &tracing::Id, values: &tracing::span::Record<'_>) {
        self.inner.record(span, values);
    }

    fn record_follows_from(&self, span: &tracing::Id, follows: &tracing::Id) {
        self.inner.record_follows_from(span, follows);
    }

    fn event(&self, event: &tracing::Event<'_>) {
        self.inner.event(event);
    }

    fn enter(&self, span: &tracing::Id) {
        self.inner.enter(span);
    }

    fn exit(&self, span: &tracing::Id) {
        self.inner.exit(span);
    }
}

pub fn setup_buffered_tracing(tag: Option<&str>) -> Arc<BufferedSubscriberLayer<Registry>> {

    let buffered_layer = match tag { 
        Some(tag) => BufferedLayer::new_with_tag(tag), 
        None      => BufferedLayer::default() 
    };

    Arc::new(BufferedSubscriberLayer {
        inner: Registry::default().with(buffered_layer.clone()),
        buffered_layer: buffered_layer.into(),
    })
}
