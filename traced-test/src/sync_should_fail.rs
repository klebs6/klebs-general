crate::ix!();

pub struct SynchronousTestShouldFail {
    return_type:    Option<Box<syn::Type>>,
    original_block: Box<syn::Block>,
    should_fail:    ShouldFailAttr,
}

impl SynchronousTestShouldFail {

    /// Creates a new builder instance.
    pub fn builder() -> SynchronousTestShouldFailBuilder {
        SynchronousTestShouldFailBuilder {
            return_type:    None,
            original_block: None,
            should_fail:    None,
        }
    }
}

impl MaybeHasExpectedFailureMessage for SynchronousTestShouldFail {

    fn expected_failure_message(&self) -> Option<Cow<'_,str>> {
        self.should_fail.expected_failure_message()
    }
}

impl HasOriginalBlock for SynchronousTestShouldFail {

    fn original_block(&self) -> &syn::Block {
        &self.original_block
    }
}

//------------------------------------
pub struct SynchronousTestShouldFailBuilder {
    return_type:    Option<Box<syn::Type>>,
    original_block: Option<Box<syn::Block>>,
    should_fail:    Option<ShouldFailAttr>,
}

impl SynchronousTestShouldFailBuilder {

    /// Sets the return type.
    pub fn return_type(mut self, return_type: Option<Box<syn::Type>>) -> Self {
        self.return_type = return_type;
        self
    }

    /// Sets the original block.
    pub fn original_block(mut self, original_block: Box<syn::Block>) -> Self {
        self.original_block = Some(original_block);
        self
    }

    /// Sets the `should_fail` attribute.
    pub fn should_fail(mut self, should_fail: ShouldFailAttr) -> Self {
        self.should_fail = Some(should_fail);
        self
    }

    /// Builds the `SynchronousTestShouldFail` instance.
    pub fn build(self) -> Result<SynchronousTestShouldFail, String> {
        Ok(SynchronousTestShouldFail {
            return_type:    self.return_type,
            original_block: self.original_block.ok_or("original_block is required")?,
            should_fail:    self.should_fail.ok_or("should_fail attribute is required")?,
        })
    }
}
