crate::ix!();

pub trait GenerateNewBlock {

    type Error;

    fn generate_new_block(&self) -> Result<Box<syn::Block>,Self::Error>;
}

impl GenerateNewBlock for TracedTestGenerator {

    type Error = TracedTestError;

    fn generate_new_block(&self) -> Result<Box<syn::Block>, Self::Error> {

        let new_use_statements               = self.use_statements_for_new_block();
        let setup_local_subscriber_and_clone = self.setup_local_subscriber();
        let setup_info_level_span            = self.setup_info_level_span();
        let captured_result                  = self.wrap_the_original_block()?;
        let result_handling_tokens           = self.result_handling_tokens();
        let flush_tracing                    = self.maybe_flush_tracing();

        // Generate the test block
        let new_block = quote! {
            {
                #new_use_statements
                #setup_local_subscriber_and_clone
                #setup_info_level_span

                let result = #captured_result;

                #flush_tracing

                #result_handling_tokens
            }
        };

        Ok(Box::new(parse_or_compile_error(new_block)?))
    }
}

impl TracedTestGenerator {

    fn maybe_flush_tracing(&self) -> TokenStream2 {

        // Check if we should flush the local subscriber
        let should_flush = if let Some(ref should_fail_attr) = self.should_fail_attr() {
            !should_fail_attr.should_trace()
        } else {
            true // Default behavior
        };

        if should_flush {
            quote! {
                local_subscriber.flush();
            }
        } else {
            quote! {}
        }
    }

    fn setup_info_level_span(&self) -> TokenStream2 {

        let test_name = self.name();

        quote!{
            let span = span!(Level::INFO, "test_trace", test_name = #test_name);
        }
    }

    fn setup_local_subscriber(&self) -> TokenStream2 {

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
