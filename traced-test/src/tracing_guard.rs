crate::ix!();

pub struct TracingGuard {
    local_subscriber: Arc<dyn Flushable + Send + Sync>,
    test_failed:      Arc<Mutex<bool>>,
    traced_test_attr: Option<TracedTestAttr>,
}

impl Drop for TracingGuard {
    fn drop(&mut self) {
        let test_failed = *self.test_failed.lock().unwrap();
        let should_flush = if test_failed {
            self.traced_test_attr
                .as_ref()
                .map(|attr| attr.should_trace_on_failure())
                .unwrap_or(true)
        } else {
            self.traced_test_attr
                .as_ref()
                .map(|attr| attr.should_trace_on_success())
                .unwrap_or(false)
        };

        if should_flush {
            self.local_subscriber.flush();
        }
    }
}

