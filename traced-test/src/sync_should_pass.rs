// ---------------- [ File: src/sync_should_pass.rs ]
crate::ix!();

pub struct SynchronousTest {
    return_type:    Option<Box<syn::Type>>,
    original_block: Box<syn::Block>,
}

impl SynchronousTest {

    /// Creates a new `SynchronousTestBuilder`.
    pub fn builder() -> SynchronousTestBuilder {
        SynchronousTestBuilder::default()
    }
}

impl MaybeHasExpectedFailureMessage for SynchronousTest {

    fn expected_failure_message(&self) -> Option<Cow<'_,str>> {
        None
    }
}

impl HasOriginalBlock for SynchronousTest {

    fn original_block(&self) -> &syn::Block {
        &self.original_block
    }
}

//-----------------------------
pub struct SynchronousTestBuilder {
    return_type:    Option<Box<syn::Type>>,
    original_block: Option<Box<syn::Block>>,
}

impl Default for SynchronousTestBuilder {
    fn default() -> Self {
        SynchronousTestBuilder {
            return_type: None,
            original_block: None,
        }
    }
}

impl SynchronousTestBuilder {
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

    /// Builds the `SynchronousTest` instance.
    pub fn build(self) -> Result<SynchronousTest, BuilderError> {
        Ok(SynchronousTest {
            return_type:    self.return_type,
            original_block: self.original_block.ok_or(BuilderError::MissingOriginalBlock)?,
        })
    }
}
