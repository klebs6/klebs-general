crate::ix!();

pub enum Test {
    Asynchronous(AsynchronousTest),
    AsynchronousShouldFail(AsynchronousTestShouldFail),
    Synchronous(SynchronousTest),
    SynchronousShouldFail(SynchronousTestShouldFail),
}

pub struct TestBuilder {
    return_type:    Option<Box<syn::Type>>,
    original_block: Option<Box<syn::Block>>,
    should_fail:    Option<ShouldFailAttr>,
    is_async:       bool,
}

impl TestBuilder {

    pub fn new() -> Self {
        Self {
            return_type:    None,
            original_block: None,
            should_fail:    None,
            is_async:       false,
        }
    }

    pub fn return_type(mut self, return_type: Option<Box<syn::Type>>) -> Self {
        self.return_type = return_type;
        self
    }

    pub fn original_block(mut self, block: &syn::Block) -> Self {
        self.original_block = Some(Box::new(block.clone()));
        self
    }

    pub fn async_test(mut self) -> Self {
        self.is_async = true;
        self
    }

    pub fn should_fail(mut self, should_fail: ShouldFailAttr) -> Self {
        self.should_fail = Some(should_fail);
        self
    }

    pub fn build(self) -> Result<Test, TracedTestError> {

        let original_block = self.original_block.ok_or(TracedTestError::MissingOriginalBlock)?;
        let return_type    = self.return_type;

        let test = match (self.is_async, self.should_fail) {
            (false, None) => {
                // Use the builder for SynchronousTest
                let synchronous_test = SynchronousTest::builder()
                    .return_type(return_type)
                    .original_block(original_block)
                    .build()?;
                Test::Synchronous(synchronous_test)
            }
            (false, Some(should_fail)) => {
                // Use the builder for SynchronousTestShouldFail
                let synchronous_test_should_fail = SynchronousTestShouldFail::builder()
                    .return_type(return_type)
                    .original_block(original_block)
                    .should_fail(should_fail)
                    .build()?;
                Test::SynchronousShouldFail(synchronous_test_should_fail)
            }
            (true, None) => {
                // Use the builder for AsynchronousTest
                let asynchronous_test = AsynchronousTest::builder()
                    .return_type(return_type)
                    .original_block(original_block)
                    .build()?;
                Test::Asynchronous(asynchronous_test)
            }
            (true, Some(should_fail)) => {
                // Use the builder for AsynchronousTestShouldFail
                let asynchronous_test_should_fail = AsynchronousTestShouldFail::builder()
                    .return_type(return_type)
                    .original_block(original_block)
                    .should_fail(should_fail)
                    .build()?;
                Test::AsynchronousShouldFail(asynchronous_test_should_fail)
            }
        };

        Ok(test)
    }
}

impl WrapBlock for Test {

    fn wrap_block(&self, generator: &TracedTestGenerator) -> TokenStream2 {
        match self {
            Test::Synchronous(test)             => test.wrap_block(generator),
            Test::SynchronousShouldFail(test)   => test.wrap_block(generator),
            Test::Asynchronous(test)            => test.wrap_block(generator),
            Test::AsynchronousShouldFail(test)  => test.wrap_block(generator),
        }
    }
}
