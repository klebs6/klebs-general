crate::ix!();

impl TracedTestGenerator {

    pub fn define_end_of_test_guard(&self) -> TokenStream2 {

        let test_name = self.name();

        quote!{
            // At the end of the test
            struct EndOfTestGuard;

            impl Drop for EndOfTestGuard {
                fn drop(&mut self) {
                    println!("{}", format!("===== END_TEST: {} =====", #test_name).bright_black());
                }
            }
        }
    }
}
