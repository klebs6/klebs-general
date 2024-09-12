crate::ix!();

pub struct TracedTestGenerator {
    orig:           ItemFn,
    name:           String,
    attrs:          Vec<syn::Attribute>,

    /// does the orig function have `should_panic`
    should_panic:   Option<ShouldPanicAttr>,

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

impl HasPanicMessage for TracedTestGenerator {

    fn panic_message(&self) -> Cow<'_,str> {
        match self.should_panic {
            Some(ref should) => should.panic_message(),
            None => Cow::from(format!("This test panicked! name={}",self.name())),
        }
    }
}

impl Named for TracedTestGenerator {
    fn name(&self) -> Cow<'_,str> {
        Cow::Borrowed(&self.name)
    }
}

impl TracedTestGenerator {

    pub fn original_block(&self) -> Box<syn::Block> {
        self.orig.block.clone()
    }

    pub fn new(orig: ItemFn) -> Result<Self,TracedTestError> {

        orig.ensure_no_test_attribute()?;

        let attrs          = orig.extract_all_attributes_except_test_attribute();
        let should_panic   = attrs.as_slice().maybe_get_should_panic_attr()?;
        let name           = orig.sig.ident.to_string();
        let is_async       = orig.is_async();
        let returns_result = orig.returns_result();

        Ok(Self {
            orig,
            name,
            attrs,
            should_panic,
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
