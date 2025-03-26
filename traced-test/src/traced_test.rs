// ---------------- [ File: src/traced_test.rs ]
crate::ix!();

#[derive(Getters,Setters,Debug)]
#[getset(get="pub",set="pub")]
pub struct TracedTestGenerator {
    orig:             ItemFn,
    name:             String,
    attrs:            Vec<syn::Attribute>,

    should_fail:      Option<ShouldFailAttr>,
    is_async:         bool,
    returns_result:   bool,

    traced_test_attr: TracedTestAttr,

    /// NEW: store final booleans after applying defaults
    show_timestamp: bool,
    show_loglevel:  bool,
}

impl TracedTestGenerator {
    pub fn from_item_fn(
        orig: syn::ItemFn,
        traced_test_attr: TracedTestAttr
    ) -> Result<Self, TracedTestError> {

        // Keep the existing check to ensure no #[test]/#[tokio::test]
        orig.ensure_no_test_attribute()?;

        // Remove logic that tried to parse `#[should_fail]` from the function’s attributes
        // Instead, just keep `extract_all_attributes_except(&[AttributeKind::TestAttr])`,
        // ignoring that old "should_fail_attr" approach.
        let attrs = orig.extract_all_attributes_except(&[AttributeKind::TestAttr]);

        // We also forbid `#[should_panic]` in combination with traced_test
        let maybe_sp = attrs.as_slice()
            .maybe_get_should_panic_attr()
            .map_err(|_| TracedTestError::ShouldPanicAttrAccessError)?;
        if maybe_sp.is_some() {
            return Err(TracedTestError::ShouldPanicAttrNotSupportedWithTracedTest);
        }

        // Now store whether or not we have "should_fail" from the macro arguments
        let is_async       = orig.is_async();
        let returns_result = orig.returns_result();

        let show_timestamp = traced_test_attr.show_timestamp().unwrap_or(true);
        let show_loglevel  = traced_test_attr.show_loglevel().unwrap_or(true);

        let name = orig.sig.ident.to_string();

        // If the user wrote: #[traced_test(should_fail(message="..."))], we’ll store that info
        let maybe_message = traced_test_attr.fail_message().clone(); // Option<String>

        Ok(Self {
            orig,
            name,
            attrs,
            // store that info so the rest of your logic can see it
            should_fail: if *traced_test_attr.should_fail() {
                // We can re-use your existing “ShouldFailAttr” struct if you prefer,
                // or simply store the message as is. For example:
                Some(ShouldFailAttr::new(maybe_message))
            } else {
                None
            },
            is_async,
            returns_result,
            traced_test_attr,
            show_timestamp,
            show_loglevel,
        })
    }
}

pub trait HasAttributes {

    fn attributes(&self) -> &[syn::Attribute];
}

impl HasAttributes for TracedTestGenerator {

    fn attributes(&self) -> &[syn::Attribute] {
        &self.attrs
    }
}

impl HasOriginalItem for TracedTestGenerator {

    type Item = ItemFn;

    fn original(&self) -> Self::Item {
        self.orig.clone()
    }
}

impl ShouldTrace for TracedTestGenerator {

    fn should_trace_on_success(&self) -> bool {
        self.traced_test_attr.should_trace_on_success()
    }

    fn should_trace_on_failure(&self) -> bool {
        self.traced_test_attr.should_trace_on_failure()
    }
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

impl HasReturnType for TracedTestGenerator {

    fn return_type(&self) -> Option<Box<syn::Type>> {
        match &self.orig.sig.output {
            syn::ReturnType::Type(_,ty) => Some(ty.clone()),
            _ => None,
        }
    }
}

impl HasReturnTypeTokens for TracedTestGenerator {

    /// Returns the return type as `TokenStream2`, defaulting to `()` if not specified.
    fn return_type_tokens(&self) -> TokenStream2 {
        if let Some(return_type) = &self.return_type() {
            quote! { #return_type }
        } else {
            quote! { () }
        }
    }
}

impl HasShouldFailAttr for TracedTestGenerator {

    fn should_fail_attr(&self) -> Option<ShouldFailAttr> {
        self.should_fail.clone()
    }
}
