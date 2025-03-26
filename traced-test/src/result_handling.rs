// ---------------- [ File: src/result_handling.rs ]
crate::ix!();

impl TracedTestGenerator {

    /// Generates the `handle_result` function definition.
    pub(crate) fn handle_result_fn_tokens(&self) -> TokenStream2 {
        quote! {
            fn handle_result<TSubscriber, T, E>(
                result:               Result<T, E>,
                expected_message:     Option<&str>,
                test_failed:          &std::sync::Arc<::std::sync::Mutex<bool>>,
                local_subscriber:     &std::sync::Arc<TSubscriber>,
                logs_already_flushed: &std::sync::Arc<::std::sync::Mutex<bool>>,
            ) -> Result<(), E>
            where
                E: std::fmt::Display,
                TSubscriber: Flushable + Send + Sync + 'static,
            {
                match result {
                    Ok(_) => {
                        if let Some(msg) = expected_message {
                            *test_failed.lock().unwrap() = true;
                            flush_logs_if_needed(local_subscriber, logs_already_flushed);
                            panic!("Expected test to fail with '{}', but it succeeded", msg);
                        } else {
                            *test_failed.lock().unwrap() = false;
                            Ok(())
                        }
                    },
                    Err(e) => {
                        *test_failed.lock().unwrap() = true;
                        if let Some(expected_message) = expected_message {
                            if e.to_string() == expected_message {
                                *test_failed.lock().unwrap() = false;
                                Ok(())
                            } else {
                                flush_logs_if_needed(local_subscriber, logs_already_flushed);
                                panic!("Unexpected error occurred: {}", e);
                            }
                        } else {
                            flush_logs_if_needed(local_subscriber, logs_already_flushed);
                            Err(e)
                        }
                    },
                }
            }
        }
    }

    pub(crate) fn result_handling_tokens(&self) -> Option<TokenStream2> {
        if !self.returns_result() {
            return None;
        }

        let handle_result_fn = self.handle_result_fn_tokens();
        let test_failed = quote! { test_failed_clone };

        if let Some(expected_message) = self.expected_failure_message() {
            // Generate tokens when `#[should_fail]` is present
            let tokens = quote! {
                #handle_result_fn
                handle_result(result, Some(#expected_message), &#test_failed, &local_subscriber, &logs_already_flushed)
            };
            Some(tokens)
        } else {
            // Generate tokens when `#[should_fail]` is not present
            let tokens = quote! {
                #handle_result_fn
                handle_result(result, None, &#test_failed, &local_subscriber, &logs_already_flushed)
            };
            Some(tokens)
        }
    }
}
