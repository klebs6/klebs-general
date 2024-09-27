crate::ix!();

pub trait WriteTokenStream {

    type Error;

    fn write_token_stream(&self) -> Result<TokenStream2,Self::Error>;
}

impl WriteTokenStream for TracedTestGenerator {

    type Error = TracedTestError;

    fn write_token_stream(&self) -> Result<TokenStream2, TracedTestError> {
        let test_attr = match self.is_async() {
            true  => quote! { #[tokio::test] },
            false => quote! { #[test] },
        };

        let mut traced_test = self.original();
        traced_test.block   = self.generate_new_block()?;

        // Ensure the function is marked as async if it was originally async
        if self.is_async() {
            traced_test.sig.asyncness = Some(syn::token::Async::default());
        }

        let attrs = self.attributes();

        let output_fn = quote! {
            #(#attrs)*
            #test_attr
            #traced_test
        };

        /*
        //remove this in production
        {
            let diag = proc_macro::Diagnostic::spanned(
                traced_test.block.span(), 
                DiagnosticLevel::Note, 
                format!("Generated block: {}", output_fn.to_string())
            );

            diag.emit();
        }
        */

        Ok(output_fn)
    }
}
