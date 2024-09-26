crate::ix!();

impl TracedTestGenerator {

    /// Generates the tracing setup tokens.
    pub fn tracing_setup_tokens(&self) -> TokenStream2 {
        let test_name = self.name();
        quote! {
            let local_subscriber = setup_buffered_tracing(Some(#test_name));
            let _guard           = tracing::subscriber::set_default(local_subscriber.clone());
            let span             = tracing::span!(tracing::Level::INFO, "test_trace", test_name = #test_name);
            let _enter           = span.enter();
        }
    }

    /// Generates the tracing teardown tokens.
    pub fn tracing_teardown_tokens(&self) -> TokenStream2 {
        quote! {
            local_subscriber.flush();
        }
    }
}
