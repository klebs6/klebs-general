crate::ix!();

pub struct TracedTestGenerator {
    orig:           ItemFn,
    name:           String,
    attrs:          Vec<syn::Attribute>,

    /// does the orig function have `should_fail`
    should_fail:   Option<ShouldFailAttr>,

    /// is the orig function async or not
    is_async:       bool,

    /// is the return type Result<T, E>
    returns_result: bool,
}

impl IsAsync for TracedTestGenerator {

    fn is_async(&self) -> bool {
        self.is_async
    }
}

impl ReturnsResult for TracedTestGenerator {

    fn returns_result(&self) -> bool {
        self.returns_result
    }
}

impl MaybeHasExpectedFailureMessage for TracedTestGenerator {

    fn expected_failure_message(&self) -> Option<Cow<'_,str>> {
        match self.should_fail {
            Some(ref should_fail_attr) => should_fail_attr.expected_failure_message(),
            None => None,
        }
    }
}

impl Named for TracedTestGenerator {

    fn name(&self) -> Cow<'_,str> {
        Cow::Borrowed(&self.name)
    }
}

impl HasOriginalBlock for TracedTestGenerator {

    fn original_block(&self) -> &syn::Block {
        &self.orig.block
    }
}

impl TracedTestGenerator {

    /// Returns the return type as `TokenStream2`, defaulting to `()` if not specified.
    pub fn return_type_tokens(&self) -> TokenStream2 {
        if let Some(return_type) = &self.return_type() {
            quote! { #return_type }
        } else {
            quote! { () }
        }
    }

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

    pub fn should_fail_attr(&self) -> Option<ShouldFailAttr> {
        self.should_fail.clone()
    }

    pub(crate) fn return_type(&self) -> Option<Box<syn::Type>> {
        match &self.orig.sig.output {
            syn::ReturnType::Type(_,ty) => Some(ty.clone()),
            _ => None,
        }
    }

    pub fn new(orig: ItemFn) -> Result<Self,TracedTestError> {

        orig.ensure_no_test_attribute()?;

        let mut attrs = orig.extract_all_attributes_except(&[AttributeKind::TestAttr]);

        let should_panic = attrs.as_slice()
            .maybe_get_should_panic_attr()
            .map_err(|e| TracedTestError::ShouldPanicAttrAccessError)?;

        if should_panic.is_some() {
            return Err(TracedTestError::ShouldPanicAttrNotSupportedWithTracedTest);
        }

        let should_fail  = attrs.as_slice().maybe_get_should_fail_attr()?;

        // Remove `should_fail` and `should_panic` attributes from `attrs`
        attrs = attrs.iter()
            .filter(|a| a.kind() != AttributeKind::ShouldFailAttr)
            .cloned()
            .collect();

        let name           = orig.sig.ident.to_string();
        let is_async       = orig.is_async();
        let returns_result = orig.returns_result();

        Ok(Self {
            orig,
            name,
            attrs,
            should_fail,
            is_async,
            returns_result,
        })
    }

    pub fn write(&self) -> Result<TokenStream2,TracedTestError> {

        let test_attr = match self.is_async {
            true  => quote! { #[tokio::test] },
            false => quote! { #[test] },
        };

        let mut traced_test = self.orig.clone();
        traced_test.block   = self.generate_new_block()?;

        let attrs       = &self.attrs;

        let output_fn = quote! {
            #(#attrs)*
            #test_attr
            #traced_test
        };

        Ok(output_fn)
    }
}
