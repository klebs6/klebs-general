// ---------------- [ File: src/async_should_fail.rs ]
crate::ix!();

pub struct AsynchronousTestShouldFail {
    return_type:    Option<Box<syn::Type>>,
    original_block: Box<syn::Block>,
    should_fail:    ShouldFailAttr,
}

impl AsynchronousTestShouldFail {

    /// Creates a new `AsynchronousTestShouldFailBuilder`.
    pub fn builder() -> AsynchronousTestShouldFailBuilder {
        AsynchronousTestShouldFailBuilder::default()
    }
}

impl HasOriginalBlock for AsynchronousTestShouldFail {

    fn original_block(&self) -> &syn::Block {
        &self.original_block
    }
}


impl MaybeHasExpectedFailureMessage for AsynchronousTestShouldFail {

    fn expected_failure_message(&self) -> Option<Cow<'_,str>> {
        self.should_fail.expected_failure_message()
    }
}

//-------------------------------
pub struct AsynchronousTestShouldFailBuilder {
    return_type:    Option<Box<syn::Type>>,
    original_block: Option<Box<syn::Block>>,
    should_fail:    Option<ShouldFailAttr>,
}

impl Default for AsynchronousTestShouldFailBuilder {
    fn default() -> Self {
        AsynchronousTestShouldFailBuilder {
            return_type:    None,
            original_block: None,
            should_fail:    None,
        }
    }
}

impl AsynchronousTestShouldFailBuilder {

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

    /// Builds the `AsynchronousTestShouldFail` instance.
    pub fn build(self) -> Result<AsynchronousTestShouldFail, BuilderError> {
        Ok(AsynchronousTestShouldFail {
            return_type:    self.return_type,
            original_block: self.original_block.ok_or(BuilderError::MissingOriginalBlock)?,
            should_fail:    self.should_fail.ok_or(BuilderError::MissingShouldFailAttr)?,
        })
    }
}
