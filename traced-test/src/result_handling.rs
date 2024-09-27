crate::ix!();

impl TracedTestGenerator {

    /// Generates the `handle_result` function definition.
    pub(crate) fn handle_result_fn_tokens(&self) -> TokenStream2 {
        quote! {
            fn handle_result<T, E>(result: Result<T, E>, expected_message: &str) -> Result<(), E>
            where
                E: std::fmt::Display,
            {
                match result {
                    Ok(_) => {
                        panic!("Expected test to fail, but it succeeded")
                    },
                    Err(ref e) if e.to_string() == expected_message => {
                        // Test passes because the expected error occurred
                        Ok(())
                    }
                    Err(e) => panic!("Unexpected error occurred: {}", e),
                }
            }
        }
    }

    pub(crate) fn result_handling_tokens(&self) -> Option<TokenStream2> {

        if !self.returns_result() {
            return None;
        }

        if let Some(expected_failure_message) = self.expected_failure_message() {

            let handle_result_fn = self.handle_result_fn_tokens();

            let tokens = quote! {
                #handle_result_fn
                handle_result(result, #expected_failure_message)
            };

            Some(tokens)

        } else {

            // No `should_fail` attribute; return the result as-is
            Some(quote!(result))
        }
    }
}
