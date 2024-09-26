crate::ix!();

impl WrapBlock for SynchronousTest {

    fn wrap_block(&self, generator: &TracedTestGenerator) -> TokenStream2 {

        let original_block   = self.original_block();
        let tracing_setup    = generator.tracing_setup_tokens();
        let tracing_teardown = generator.tracing_teardown_tokens();

        if generator.returns_result() {
            let return_type_tokens = generator.return_type_tokens();

            quote! {
                {
                    #tracing_setup
                    let result = (|| -> #return_type_tokens { #original_block })();
                    #tracing_teardown
                    result
                }
            }
        } else {
            quote! {
                {
                    #tracing_setup
                    #original_block
                    #tracing_teardown
                }
            }
        }
    }
}

impl WrapBlock for SynchronousTestShouldFail {

    fn wrap_block(&self, generator: &TracedTestGenerator) -> TokenStream2 {

        let original_block   = self.original_block();
        let expected_message = self.expected_failure_message().unwrap_or(Cow::Borrowed(""));
        let tracing_setup    = generator.tracing_setup_tokens();
        let tracing_teardown = generator.tracing_teardown_tokens();

        if generator.returns_result() {
            let return_type_tokens = generator.return_type_tokens();

            quote! {
                {
                    #tracing_setup
                    let result = (|| -> #return_type_tokens { #original_block })();
                    #tracing_teardown

                    result
                }
            }
        } else {
            let handle_panic_fn = generator.handle_panic_fn_tokens();

            quote! {
                {
                    #tracing_setup
                    let result = std::panic::catch_unwind(|| {
                        #original_block
                    });
                    #tracing_teardown

                    #handle_panic_fn

                    match result {
                        Ok(_) => panic!("[BBB] Expected test to fail, but it succeeded"),
                        Err(err) => {
                            handle_panic(err, #expected_message);
                        }
                    }
                }
            }
        }
    }
}


/*
//todo!("sync and should panic");

/*
|| no method named `is_err` found for unit type `()` in the current scope
||    |
|| 17 | #[traced_test]
||    | ^^^^^^^^^^^^^^ method not found in `()`
||    |
*/

let expected_panic_message = should.panic_message();

quote! {
    {
        let result = std::panic::catch_unwind(|| {
            span.in_scope(|| {
                #original_block
            })
        });

        if let Err(panic_info) = result {
            let panic_message = panic_info.downcast_ref::<&str>().unwrap_or(&"");
            local_subscriber.flush();
            assert!(panic_message.contains(#expected_panic_message), "Expected panic message: {}", #expected_panic_message);
        }
    }
}

// Sync case without `should_panic`
// this case seems like it works
quote! {
    {
        let task = || {
            span.in_scope(|| #original_block)
        };

        std::panic::catch_unwind(AssertUnwindSafe(task))  // Unwrap to propagate any panics
    }
}
*/
