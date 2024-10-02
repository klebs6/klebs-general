crate::ix!();

impl TracedTestGenerator {

    pub fn define_flush_logs_if_needed(&self) -> TokenStream2 {
        quote!{
            // Define flush_logs_if_needed function
            pub fn flush_logs_if_needed<TSubscriber>(
                local_subscriber: &std::sync::Arc<TSubscriber>,
                logs_already_flushed: &std::sync::Arc<std::sync::Mutex<bool>>,
            )
            where
                TSubscriber: Flushable + Send + Sync + 'static,
            {
                let mut flushed = logs_already_flushed.lock().unwrap();
                if !*flushed {
                    local_subscriber.flush();
                    *flushed = true;
                }
            }
        }
    }
}
