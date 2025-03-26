// ---------------- [ File: src/panic_handler.rs ]
crate::ix!();

pub trait CheckForAndRetrieveTheUniqueShouldPanicAttr {

    /// Returns `Some(ShouldPanicAttr)` if the attribute is found.
    /// Returns `Err` if there's a duplicate or any other error while parsing.
    fn maybe_get_should_panic_attr(&self)
        -> Result<Option<ShouldPanicAttr>, ShouldPanicAttrError>;
}

pub trait MaybeHasPanicMessage {

    fn panic_message(&self)
        -> Option<Cow<'_, str>>;
}

impl TracedTestGenerator {

    pub fn handle_panic_fn_tokens(&self) -> TokenStream2 {
        quote! {
            fn handle_panic<TSubscriber>(
                err:                  Box<dyn std::any::Any + Send>,
                expected_message:     Option<&str>,
                test_failed:          &std::sync::Arc<std::sync::Mutex<bool>>,
                local_subscriber:     &std::sync::Arc<TSubscriber>,
                logs_already_flushed: &std::sync::Arc<std::sync::Mutex<bool>>,
            )
            where
                TSubscriber: Flushable + Send + Sync + 'static,
            {
                let panic_message = if let Some(s) = err.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = err.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown panic message".to_string()
                };
                if let Some(expected) = expected_message {
                    if panic_message == expected {
                        *test_failed.lock().unwrap() = false;
                    } else {
                        *test_failed.lock().unwrap() = true;
                        flush_logs_if_needed(local_subscriber, logs_already_flushed);
                        panic!("Unexpected panic occurred: {}", panic_message);
                    }
                } else {
                    // No expected message; accept any panic
                    *test_failed.lock().unwrap() = false;
                }
            }
        }
    }
}
