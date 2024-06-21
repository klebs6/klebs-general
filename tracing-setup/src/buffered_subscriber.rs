crate::ix!();

pub trait BufferedSubscriber: Subscriber + Flushable {}

pub struct BufferedSubscriberLayer<S> {
    pub(crate) inner: tracing_subscriber::layer::Layered<BufferedLayer, S>,
    pub(crate) buffered_layer: Arc<BufferedLayer>,
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
