// ---------------- [ File: src/wrap_async_test.rs ]
crate::ix!();

impl WrapBlock for AsynchronousTest {
    fn wrap_block(&self, generator: &TracedTestGenerator) -> TokenStream2 {
        let original_block = self.original_block();
        let test_failed = quote! { test_failed_clone }; // Reference to test_failed_clone

        if *generator.returns_result() {
            let return_type_tokens = generator.return_type_tokens();
            let result_handling_tokens = generator.result_handling_tokens().unwrap();

            quote! {
                {
                    use tracing::Instrument;
                    // Include flush_logs_if_needed function if not already included
                    // #flush_logs_fn

                    let handle = tokio::spawn(async {
                        #original_block
                    }.instrument(tracing::Span::current()));

                    let result = handle.await;
                    let result = match result {
                        Ok(val) => val,
                        Err(err) => {
                            if err.is_panic() {
                                // Flush logs before resuming panic
                                *#test_failed.lock().unwrap() = true;
                                flush_logs_if_needed(&local_subscriber, &logs_already_flushed);
                                std::panic::resume_unwind(err.into_panic())
                            } else {
                                panic!("Task was cancelled");
                            }
                        },
                    };

                    #result_handling_tokens
                }
            }
        } else {
            quote! {
                {
                    use tracing::Instrument;
                    // Include flush_logs_if_needed function if not already included
                    // #flush_logs_fn

                    let handle = tokio::spawn(async {
                        #original_block
                    }.instrument(tracing::Span::current()));

                    let result = handle.await;
                    match result {
                        Ok(_) => {
                            // Test passed; set test_failed to false
                            *#test_failed.lock().unwrap() = false;
                        },
                        Err(err) => {
                            if err.is_panic() {
                                // Flush logs before resuming panic
                                *#test_failed.lock().unwrap() = true;
                                flush_logs_if_needed(&local_subscriber, &logs_already_flushed);
                                std::panic::resume_unwind(err.into_panic())
                            } else {
                                panic!("Task was cancelled");
                            }
                        },
                    }
                }
            }
        }
    }
}

impl WrapBlock for AsynchronousTestShouldFail {

    fn wrap_block(&self, generator: &TracedTestGenerator) -> TokenStream2 {
        let original_block       = self.original_block();
        let expected_message     = self.expected_failure_message();
        let expected_message_str = expected_message.clone().unwrap_or(Cow::Borrowed(""));
        let handle_panic_fn      = generator.handle_panic_fn_tokens();

        let expected_message_tokens = match expected_message {
            Some(ref msg) => quote! { Some(#msg) },
            None => quote! { None },
        };

        let test_failed = quote! { test_failed_clone };

        if *generator.returns_result() {
            let return_type_tokens = generator.return_type_tokens();

            quote! {
                {
                    use tracing::Instrument;
                    use std::borrow::Cow;
                    #handle_panic_fn
                    // Include flush_logs_if_needed function if not already included
                    // #flush_logs_fn

                    let task = async move {
                        let result: #return_type_tokens = #original_block;
                        result
                    }.instrument(tracing::Span::current());

                    let handle = tokio::spawn(task);

                    let result = handle.await;

                    let final_result: #return_type_tokens = match result {
                        Ok(val) => val,
                        Err(err) => {
                            if err.is_panic() {
                                let panic_err = err.into_panic();
                                flush_logs_if_needed(&local_subscriber, &logs_already_flushed);
                                handle_panic(
                                    panic_err,
                                    #expected_message_tokens,
                                    &#test_failed,
                                    &local_subscriber,
                                    &logs_already_flushed,
                                );
                                // Return the expected error
                                Err(#expected_message_str.to_string())
                            } else {
                                panic!("Task was cancelled");
                            }
                        },
                    };

                    final_result
                }
            }
        } else {
            quote! {
                {
                    use tracing::Instrument;
                    use std::borrow::Cow;
                    #handle_panic_fn
                    // Include flush_logs_if_needed function if not already included
                    // #flush_logs_fn

                    let handle = tokio::spawn(async {
                        #original_block
                    }.instrument(tracing::Span::current()));

                    let result = handle.await;

                    match result {
                        Ok(_) => {
                            *#test_failed.lock().unwrap() = true;
                            flush_logs_if_needed(&local_subscriber, &logs_already_flushed);
                            panic!("Expected test to fail, but it succeeded");
                        },
                        Err(err) => {
                            if err.is_panic() {
                                let panic_err = err.into_panic();
                                flush_logs_if_needed(&local_subscriber, &logs_already_flushed);
                                handle_panic(
                                    panic_err,
                                    #expected_message_tokens,
                                    &#test_failed,
                                    &local_subscriber,
                                    &logs_already_flushed,
                                );
                            } else {
                                panic!("Task was cancelled");
                            }
                        },
                    }
                }
            }
        }
    }
}
