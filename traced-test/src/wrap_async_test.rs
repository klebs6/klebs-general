crate::ix!();

impl WrapBlock for AsynchronousTest {

    fn wrap_block(&self, generator: &TracedTestGenerator) -> TokenStream2 {

        let original_block   = self.original_block();
        let tracing_setup    = generator.tracing_setup_tokens();
        let tracing_teardown = generator.tracing_teardown_tokens();

        if generator.returns_result() {
            let return_type_tokens = generator.return_type_tokens();

            quote! {
                {
                    #tracing_setup
                    let result = async move { #original_block }.await;
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

impl WrapBlock for AsynchronousTestShouldFail {

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

                    let result: #return_type_tokens = async move { #original_block }.await;

                    #tracing_teardown

                    result
                }
            }


        } else {

            let handle_panic_fn = generator.handle_panic_fn_tokens();

            quote! {
                {
                    #tracing_setup
                    let result = tokio::spawn(async { #original_block }).await;
                    #tracing_teardown

                    #handle_panic_fn

                    match result {
                        Ok(_) => panic!("[AAA] Expected test to fail, but it succeeded"),
                        Err(err) => {
                            if err.is_panic() {
                                let panic_err = err.into_panic();
                                handle_panic(panic_err, #expected_message);
                            } else {
                                panic!("Task was cancelled: {:?}", err);
                            }
                        }
                    }
                }
            }
        }
    }
}

// this is what we had earlier when it should panic
//
/*
todo!("async and should panic");
let expected_panic_message = should.panic_message();

quote! {{

    use tokio::task::JoinError;

    let _enter = span.enter();

    // Define the async task separately
    let async_task = async move {
        // Run the original async block
        #original_block
    };

    // Spawn the async task and wait for it to complete
    let result: Result<#task_return_type,JoinError> = tokio::task::spawn(async_task).await;

    if result.is_err() {
        panic!("Test panicked unexpectedly!");
    }

    let result: #task_return_type = result.unwrap();

    /*

    if let Err(panic_info) = result {

        let panic_message = panic_info.downcast_ref::<&str>().unwrap_or(&"");

        assert!(
            panic_message.contains(#expected_panic_message), 
            "Expected panic message: {}", 
            #expected_panic_message
        );
    }

    */

    result
}}

*/

/*
//todo!("async without should panic");
quote! {
    {
        let _enter = span.enter();

        // Define the async task separately
        let async_task = async move {
            #original_block
        };

        // Spawn the async task and wait for it to complete
        tokio::task::spawn(async_task).await
    }
}
*/
