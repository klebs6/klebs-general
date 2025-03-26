// ---------------- [ File: src/tracing_guard.rs ]
crate::ix!();

impl TracedTestGenerator {

    pub fn define_tracing_guard(&self) -> TokenStream2 {
        quote!{
            // TracingGuard struct and implementation
            #[derive(Debug)]
            struct TracingGuard<TSubscriber>
            where
                TSubscriber: Flushable + Send + Sync + 'static,
            {
                local_subscriber: ::std::sync::Arc<TSubscriber>,
                test_failed: ::std::sync::Arc<::std::sync::Mutex<bool>>,
                should_trace_on_success: bool,
                should_trace_on_failure: bool,
                logs_already_flushed: ::std::sync::Arc<::std::sync::Mutex<bool>>,
            }

            impl<TSubscriber> ShouldTrace for TracingGuard<TSubscriber>
            where
                TSubscriber: Flushable + Send + Sync + 'static,
            {
                fn should_trace_on_success(&self) -> bool {
                    self.should_trace_on_success
                }

                fn should_trace_on_failure(&self) -> bool {
                    self.should_trace_on_failure
                }
            }

            impl<TSubscriber> Drop for TracingGuard<TSubscriber>
            where
                TSubscriber: Flushable + Send + Sync + 'static,
            {
                fn drop(&mut self) {
                    let test_failed = *self.test_failed.lock().unwrap();
                    let mut flushed = self.logs_already_flushed.lock().unwrap();
                    let should_flush = if test_failed {
                        self.should_trace_on_failure
                    } else {
                        self.should_trace_on_success
                    };

                    if should_flush && !*flushed {
                        self.local_subscriber.flush();
                        *flushed = true;
                    }
                }
            }

        }
    }
}
