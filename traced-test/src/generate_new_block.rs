crate::ix!();

pub trait GenerateNewBlock {

    type Error;

    fn generate_new_block(&self) -> Result<Box<syn::Block>,Self::Error>;
}

impl GenerateNewBlock for TracedTestGenerator {

    type Error = TracedTestError;

    fn generate_new_block(&self) -> Result<Box<syn::Block>,Self::Error> {

        let new_use_statements               = self.use_statements_for_new_block();
        let setup_local_subscriber_and_clone = self.setup_local_subscriber_and_clone();
        let setup_info_level_span            = self.setup_info_level_span();
        let captured_result                  = self.captured_result_from_the_original_block();
        let maybe_unwrap_result              = self.maybe_unwrap_result();

        // Generate the test block
        let new_block = quote!{{

            #new_use_statements
            #setup_local_subscriber_and_clone
            #setup_info_level_span

            let result = #captured_result ;

            if result.is_err() {
                local_subscriber.flush();
            }

            #maybe_unwrap_result
        }};

        Ok(Box::new(parse_or_compile_error(new_block)?))
    }
}

impl TracedTestGenerator {

    fn maybe_unwrap_result(&self) -> TokenStream2 {

        let panic_message = self.panic_message();

        match self.returns_result() {
            true => {
                quote!({
                    match result {
                        Ok(res) => {
                            local_subscriber.flush();
                            res // Ensure the result is returned properly
                        }
                        Err(_) => {
                            local_subscriber.flush();
                            panic!("{}", #panic_message); // Explicit panic on unexpected error
                        }
                    }
                })
            },
            false => {
                quote!{{
                    if result.is_err() {
                        local_subscriber.flush();
                        panic!("{}", #panic_message); // Explicit panic on unexpected error
                    }
                }}
            }
        }
    }

    fn captured_result_from_the_original_block(&self) -> TokenStream2 {

        let original_block = self.original_block();

        match self.is_async() {

            true => quote!{{

                let task = async move {
                    span.in_scope(|| #original_block)
                };
                task::spawn(task).await

            }},

            false => quote!{{

                let task = || {
                    span.in_scope(|| #original_block)
                };

                std::panic::catch_unwind(AssertUnwindSafe(task))
            }}
        }
    }

    fn setup_info_level_span(&self) -> TokenStream2 {

        let test_name = self.name();

        quote!{
            let span = span!(Level::INFO, "test_trace", test_name = #test_name);
        }
    }

    fn setup_local_subscriber_and_clone(&self) -> TokenStream2 {

        let test_name = self.name();

        quote!{
            let local_subscriber = setup_buffered_tracing(Some(#test_name));
            let local_subscriber_clone = local_subscriber.clone();
            let _guard = tracing::subscriber::set_default(local_subscriber.clone());
        }
    }

    fn use_statements_for_new_block(&self) -> TokenStream2 {

        let mut use_statements = TokenStream2::new();

        use_statements.extend(quote!{ use tracing::*; });
        use_statements.extend(quote!{ use std::panic::AssertUnwindSafe; });

        if self.is_async() {
            use_statements.extend(quote!{ use tokio::task; });
        }

        use_statements
    }
}
