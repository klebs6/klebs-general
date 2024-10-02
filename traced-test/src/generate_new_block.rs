crate::ix!();

pub trait GenerateNewBlock {

    type Error;

    fn generate_new_block(&self) -> Result<Box<syn::Block>,Self::Error>;
}

impl GenerateNewBlock for TracedTestGenerator {

    type Error = TracedTestError;

    fn generate_new_block(&self) -> Result<Box<syn::Block>, Self::Error> {
        let use_statements         = self.use_statements_for_new_block();
        let captured_result        = self.wrap_the_original_block()?;
        let result_handling_tokens = self.result_handling_tokens();
        let tracing_setup          = self.tracing_setup_tokens();

        let new_block = if self.is_async() {
            quote! {
                {
                    #use_statements

                    #tracing_setup

                    // Async context allows `await`
                    let result = {
                        #captured_result
                    };

                    #result_handling_tokens
                }
            }
        } else {
            quote! {
                {
                    #use_statements

                    #tracing_setup

                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        #captured_result
                    }));

                    match result {
                        Ok(result) => { #result_handling_tokens },
                        Err(err) => std::panic::resume_unwind(err),
                    }
                }
            }
        };

        Ok(Box::new(parse_or_compile_error(new_block)?))
    }
}
