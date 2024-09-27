crate::ix!();

pub struct AsynchronousTest {
    return_type:    Option<Box<syn::Type>>,
    original_block: Box<syn::Block>,
}

impl AsynchronousTest {

    /// Creates a new `AsynchronousTestBuilder`.
    pub fn builder() -> AsynchronousTestBuilder {
        AsynchronousTestBuilder::default()
    }
}

impl HasOriginalBlock for AsynchronousTest {

    fn original_block(&self) -> &syn::Block {
        &self.original_block
    }
}


impl MaybeHasExpectedFailureMessage for AsynchronousTest {

    fn expected_failure_message(&self) -> Option<Cow<'_,str>> {
        None
    }
}

//---------------------------
pub struct AsynchronousTestBuilder {
    return_type:    Option<Box<syn::Type>>,
    original_block: Option<Box<syn::Block>>,
}

impl Default for AsynchronousTestBuilder {

    fn default() -> Self {
        AsynchronousTestBuilder {
            return_type:    None,
            original_block: None,
        }
    }
}

impl AsynchronousTestBuilder {
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

    /// Builds the `AsynchronousTest` instance.
    pub fn build(self) -> Result<AsynchronousTest, BuilderError> {
        Ok(AsynchronousTest {
            return_type:    self.return_type,
            original_block: self.original_block.ok_or(BuilderError::MissingOriginalBlock)?,
        })
    }
}
