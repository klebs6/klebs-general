crate::ix!();

impl TracedTestGenerator {

    /// Generates the tracing setup tokens.
    pub fn tracing_setup_tokens(&self) -> TokenStream2 {
        let test_name    = self.name();

        quote! {
            let local_subscriber = setup_buffered_tracing(Some(#test_name));
            let _guard           = tracing::subscriber::set_default(local_subscriber.clone());
            let span             = tracing::span!(tracing::Level::INFO, "test_trace", test_name = #test_name);
            let _enter           = span.enter();

            // Define the TracingGuard struct
            struct TracingGuard {
                local_subscriber: ::std::sync::Arc<dyn Flushable + Send + Sync>,
            }

            // Implement Drop for TracingGuard to flush logs
            impl Drop for TracingGuard {
                fn drop(&mut self) {
                    self.local_subscriber.flush();
                }
            }

            // Instantiate TracingGuard to ensure logs are flushed on drop
            let _tracing_guard = TracingGuard {
                local_subscriber: local_subscriber.clone(),
            };
        }
    }
}
