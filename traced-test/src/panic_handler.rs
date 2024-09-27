crate::ix!();

pub trait CheckForAndRetrieveTheUniqueShouldPanicAttr {

    /// Returns `Some(ShouldPanicAttr)` if the attribute is found.
    /// Returns `Err` if there's a duplicate or any other error while parsing.
    fn maybe_get_should_panic_attr(&self)
        -> Result<Option<ShouldPanicAttr>, ShouldPanicAttrError>;
}

pub trait MaybeHasPanicMessage {

    fn panic_message(&self)
        -> Option<Cow<'_, str>>;
}

impl TracedTestGenerator {

    /// Generates the `handle_panic` function definition.
    pub fn handle_panic_fn_tokens(&self) -> TokenStream2 {
        quote! {
            fn handle_panic(err: Box<dyn std::any::Any + Send>, expected_message: &str) {
                let panic_message = if let Some(s) = err.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = err.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown panic message".to_string()
                };
                if panic_message == expected_message {
                    // Test passes
                } else {
                    panic!("Unexpected panic occurred: {}", panic_message);
                }
            }
        }
    }
}
