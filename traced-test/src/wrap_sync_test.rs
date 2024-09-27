crate::ix!();

impl WrapBlock for SynchronousTest {

    fn wrap_block(&self, generator: &TracedTestGenerator) -> TokenStream2 {

        let original_block = self.original_block();

        if generator.returns_result() {

            let return_type_tokens = generator.return_type_tokens();

            quote! {
                {
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        (|| -> #return_type_tokens { #original_block })()
                    }));

                    match result {
                        Ok(val) => val,
                        Err(err) => std::panic::resume_unwind(err),
                    }
                }
            }
        } else {
            quote! {
                {
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        #original_block
                    }));

                    match result {
                        Ok(_) => {},
                        Err(err) => std::panic::resume_unwind(err),
                    }
                }
            }
        }
    }
}

impl WrapBlock for SynchronousTestShouldFail {

    fn wrap_block(&self, generator: &TracedTestGenerator) -> TokenStream2 {

        let original_block   = self.original_block();
        let expected_message = self.expected_failure_message().unwrap_or(Cow::Borrowed(""));
        let handle_panic_fn  = generator.handle_panic_fn_tokens();

        if generator.returns_result() {

            let return_type_tokens = generator.return_type_tokens();

            quote! {
                {
                    #handle_panic_fn

                    let result: #return_type_tokens = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        (|| -> #return_type_tokens { #original_block })()
                    }))
                    .unwrap_or_else(|err| {
                        handle_panic(err, #expected_message);
                        // Return an Err variant with the expected message
                        Err(#expected_message.to_string())

                    });

                    result
                }
            }

        } else {

            quote! {
                {
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        #original_block
                    }));

                    #handle_panic_fn

                    match result {
                        Ok(_) => panic!("Expected test to fail, but it succeeded"),
                        Err(err) => {
                            handle_panic(err, #expected_message);
                        }
                    }
                }
            }
        }
    }
}
