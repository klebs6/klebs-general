crate::ix!();

impl TracedTestGenerator {

    pub fn define_backtrace_guard(&self) -> TokenStream2 {

        quote!{

            enum BacktraceMode {
                Disabled,   // Corresponds to "0" or unset
                Enabled,    // Corresponds to "1"
                Full,       // Corresponds to "full"
            }

            impl BacktraceMode {
                pub fn from_env() -> Self {
                    match std::env::var("RUST_BACKTRACE").as_deref() {
                        Ok("1")    => BacktraceMode::Enabled,
                        Ok("full") => BacktraceMode::Full,

                        // Treat any other value, including unset, as Disabled
                        _          => BacktraceMode::Disabled, 
                    }
                }
            }

            struct BacktraceGuard {
                previous_rust_backtrace_value: BacktraceMode,
                test_name:                     String,
            }

            impl BacktraceGuard {

                /// Constructor for BacktraceGuard that sets
                /// the previous backtrace state and updates
                /// RUST_BACKTRACE to "0" to disable it for
                /// the test.
                ///
                pub fn new(test_name: String) -> Self {

                    let previous_rust_backtrace_value = BacktraceMode::from_env();

                    // Disable backtrace by default for the test run
                    std::env::set_var("RUST_BACKTRACE", "0");

                    BacktraceGuard {
                        previous_rust_backtrace_value,
                        test_name,
                    }
                }

                pub fn maybe_print_backtrace(&self) {
                    match self.previous_rust_backtrace_value {
                        BacktraceMode::Enabled | BacktraceMode::Full => {
                            // Print backtrace since it was enabled or set to full
                            self.print_backtrace();
                        }
                        BacktraceMode::Disabled => {
                            // Do not print the backtrace
                        }
                    }
                }

                pub fn print_backtrace(&self) {

                    let header    = format!("===== Backtrace for test: {} =====", self.test_name);
                    let backtrace = format!("{:#?}", std::backtrace::Backtrace::force_capture());

                    println!("{}", header.bright_black());
                    println!("{}", backtrace.bright_black());
                }

                /// Restore the previous RUST_BACKTRACE value
                pub fn restore_previous_rust_backtrace_value(&self) {

                    match &self.previous_rust_backtrace_value {
                        BacktraceMode::Enabled  => std::env::set_var("RUST_BACKTRACE", "1"),
                        BacktraceMode::Full     => std::env::set_var("RUST_BACKTRACE", "full"),
                        BacktraceMode::Disabled => std::env::set_var("RUST_BACKTRACE", "0"), // Reset to disabled state
                    }
                }
            }

            impl Drop for BacktraceGuard {

                fn drop(&mut self) {

                    // Check if backtrace was previously enabled
                    if std::thread::panicking() {
                        self.maybe_print_backtrace();
                    }

                    self.restore_previous_rust_backtrace_value();
                }
            }
        }
    }
}
