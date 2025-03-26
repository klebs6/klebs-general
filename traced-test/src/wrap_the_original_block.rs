// ---------------- [ File: src/wrap_the_original_block.rs ]
crate::ix!();

pub trait WrapBlock {

    fn wrap_block(&self,generator: &TracedTestGenerator) -> TokenStream2;
}

impl TracedTestGenerator {

    pub(crate) fn wrap_the_original_block(&self) 
        -> Result<TokenStream2, TracedTestError> 
    {
        let mut builder = TestBuilder::new()
            .return_type(self.return_type())
            .original_block(self.original_block());

        if *self.is_async() {
            builder = builder.async_test();
        }

        if let Some(should_fail) = self.should_fail_attr() {
            builder = builder.should_fail(should_fail);
        }

        let test = builder.build()?;

        Ok(test.wrap_block(self))
    }
}
