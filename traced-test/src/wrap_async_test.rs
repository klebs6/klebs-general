crate::ix!();

impl WrapBlock for AsynchronousTest {

    fn wrap_block(&self, generator: &TracedTestGenerator) -> TokenStream2 {

        let original_block = self.original_block();

        if generator.returns_result() {

            let return_type_tokens = generator.return_type_tokens();

            quote! {
                {
                    use tracing::Instrument;

                    let handle = tokio::spawn(async {
                        #original_block
                    }.instrument(tracing::Span::current()));

                    let result = handle.await;
                    match result {
                        Ok(val) => val,
                        Err(err) => {
                            if err.is_panic() {
                                std::panic::resume_unwind(err.into_panic())
                            } else {
                                panic!("Task was cancelled");
                            }
                        },
                    }
                }
            }

        } else {

            quote! {
                {
                    use tracing::Instrument;

                    let handle = tokio::spawn(async {
                        #original_block
                    }.instrument(tracing::Span::current()));

                    let result = handle.await;
                    match result {
                        Ok(_) => {},
                        Err(err) => {
                            if err.is_panic() {
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
        let original_block    = self.original_block();
        let expected_message  = self.expected_failure_message().unwrap_or(Cow::Borrowed(""));
        let handle_panic_fn   = generator.handle_panic_fn_tokens();

        if generator.returns_result() {
            let return_type_tokens = generator.return_type_tokens();

            let tokens = quote! {
                {
                    use tracing::Instrument;

                    // Specify the return type of the async block
                    let task = async move {
                        // Explicitly specify the result type
                        let result: #return_type_tokens = #original_block;
                        result
                    }
                    .instrument(tracing::Span::current());

                    let handle = tokio::spawn(task);

                    // Await the handle, which gives us a Result of the task's output or a JoinError
                    let result = handle.await;

                    #handle_panic_fn

                    let final_result: #return_type_tokens = match result {
                        Ok(val)  => val,
                        Err(err) => {
                            // The task panicked or was cancelled
                            if err.is_panic() {
                                let panic_err = err.into_panic();
                                handle_panic(panic_err, #expected_message);
                                // Return the expected error
                                Err(#expected_message.to_string())
                            } else {
                                panic!("Task was cancelled");
                            }
                        },
                    };

                    final_result
                }
            };

            // Optionally, print the generated tokens for debugging
            // println!("Generated tokens: {}", tokens);

            tokens
        } else {
            // Existing code for non-Result return types remains unchanged
            quote! {
                {
                    use tracing::Instrument;

                    let handle = tokio::spawn(async {
                        #original_block
                    }.instrument(tracing::Span::current()));

                    let result = handle.await;

                    #handle_panic_fn

                    match result {
                        Ok(_) => panic!("Expected test to fail, but it succeeded"),
                        Err(err) => {
                            if err.is_panic() {
                                let panic_err = err.into_panic();
                                handle_panic(panic_err, #expected_message);
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
