// ---------------- [ File: src/wrap_sync_test.rs ]
crate::ix!();

impl WrapBlock for SynchronousTest {
    fn wrap_block(&self, generator: &TracedTestGenerator) -> TokenStream2 {
        let original_block = self.original_block();
        let test_failed = quote! { test_failed_clone };

        if *generator.returns_result() {
            let return_type_tokens = generator.return_type_tokens();
            let result_handling_tokens = generator.result_handling_tokens().unwrap();

            quote! {
                {
                    let result: #return_type_tokens = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        (|| -> #return_type_tokens { #original_block })()
                    }))
                    .unwrap_or_else(|err| {
                        *#test_failed.lock().unwrap() = true;
                        flush_logs_if_needed(&local_subscriber, &logs_already_flushed);

                        // Panic message and backtrace will be printed after logs
                        std::panic::resume_unwind(err)
                    });

                    #result_handling_tokens
                }
            }
        } else {
            quote! {
                {
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        #original_block
                    }));

                    match result {
                        Ok(_) => {
                            *#test_failed.lock().unwrap() = false;
                        },
                        Err(err) => {
                            *#test_failed.lock().unwrap() = true;
                            flush_logs_if_needed(&local_subscriber, &logs_already_flushed);
                            std::panic::resume_unwind(err)
                        },
                    }
                }
            }
        }
    }
}


impl WrapBlock for SynchronousTestShouldFail {

    fn wrap_block(&self, generator: &TracedTestGenerator) -> TokenStream2 {

        let original_block       = self.original_block();
        let expected_message     = self.expected_failure_message();
        let expected_message_str = expected_message.clone().unwrap_or(Cow::Borrowed(""));
        let handle_panic_fn      = generator.handle_panic_fn_tokens();

        let expected_message_tokens = match expected_message {
            Some(ref msg) => quote! { Some(#msg) },
            None => quote! { None },
        };

        // Reference to test_failed_clone
        let test_failed = quote! { test_failed_clone };

        if *generator.returns_result() {

            let return_type_tokens = generator.return_type_tokens();

            quote! {
                {
                    use std::borrow::Cow;
                    #handle_panic_fn

                    let result: #return_type_tokens = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        (|| -> #return_type_tokens { #original_block })()
                    }))
                    .unwrap_or_else(|err| {
                        flush_logs_if_needed(&local_subscriber, &logs_already_flushed);
                        handle_panic(err, #expected_message_tokens, &#test_failed, &local_subscriber, &logs_already_flushed);
                        // Return an Err variant with the expected message
                        Err(#expected_message_str.to_string())

                    });

                    result
                }
            }

        } else {

            quote! {
                {
                    use std::borrow::Cow;
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        #original_block
                    }));

                    #handle_panic_fn

                    match result {
                        Ok(_) => {
                            // Test did not fail as expected; set test_failed to true
                            *#test_failed.lock().unwrap() = true;
                            panic!("Expected test to fail, but it succeeded");
                        },
                        Err(err) => {
                            flush_logs_if_needed(&local_subscriber, &logs_already_flushed);
                            handle_panic(err, #expected_message_tokens, &#test_failed, &local_subscriber, &logs_already_flushed);
                        }
                    }
                }
            }
        }
    }
}
