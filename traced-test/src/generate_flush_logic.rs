crate::ix!();

impl TracedTestGenerator {

    pub fn generate_should_flush_logic(&self) -> TokenStream2 {

        let should_trace_on_success = self.should_trace_on_success();
        let should_trace_on_failure = self.should_trace_on_failure();

        quote! {
            |test_failed: bool| {
                if test_failed {
                    #should_trace_on_failure
                } else {
                    #should_trace_on_success
                }
            }
        }
    }
}



