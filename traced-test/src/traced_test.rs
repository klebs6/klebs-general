crate::ix!();

#[derive(Debug)]
pub struct TracedTestGenerator {
    orig:             ItemFn,
    name:             String,
    attrs:            Vec<syn::Attribute>,

    /// does the orig function have `should_fail`
    should_fail:      Option<ShouldFailAttr>,

    /// is the orig function async or not
    is_async:         bool,

    /// is the return type Result<T, E>
    returns_result:   bool,
    traced_test_attr: TracedTestAttr,
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

impl TracedTestGenerator {

    pub fn from_item_fn(orig: ItemFn, traced_test_attr: TracedTestAttr) -> Result<Self, TracedTestError> {

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

        if traced_test_attr.specified() {
            todo!("force trace on/off attribute still must be implemented");
        }

        let generator = Self {
            orig,
            name,
            attrs,
            should_fail,
            is_async,
            returns_result,
            traced_test_attr,
        };

        //println!("created generator: {:#?}", generator);

        Ok(generator)
    }
}
