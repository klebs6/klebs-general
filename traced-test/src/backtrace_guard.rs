crate::ix!();

impl TracedTestGenerator {

    pub fn define_backtrace_guard(&self) -> TokenStream2 {
        quote!{
            struct BacktraceGuard {
                previous_value: Option<String>,
                test_name: String,
            }

            impl Drop for BacktraceGuard {
                fn drop(&mut self) {
                    if std::thread::panicking() {
                        // A panic is in progress; capture and print the backtrace
                        println!("{}", format!("===== Backtrace for test: {} =====", self.test_name).bright_black());
                        println!("{}", format!("{:#?}", std::backtrace::Backtrace::force_capture()).bright_black());
                    }

                    // Restore the previous RUST_BACKTRACE value
                    if let Some(val) = &self.previous_value {
                        std::env::set_var("RUST_BACKTRACE", val);
                    } else {
                        std::env::remove_var("RUST_BACKTRACE");
                    }
                }
            }
        }
    }
}
