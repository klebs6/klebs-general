// ---------------- [ File: src/backtrace_guard.rs ]
crate::ix!();

impl TracedTestGenerator {

    /// Defines (via code generation) the `BacktraceGuard` type and its helpers, now accepting the
    /// additional `enable_backtrace: bool` parameter to fix the two-argument call.
    pub fn define_backtrace_guard(&self) -> TokenStream2 {
        quote! {
            #[derive(Debug)]
            enum BacktraceMode {
                Disabled,
                Enabled,
                Full,
            }

            impl BacktraceMode {
                pub fn from_env() -> Self {
                    match std::env::var("RUST_BACKTRACE").as_deref() {
                        Ok("1") => BacktraceMode::Enabled,
                        Ok("full") => BacktraceMode::Full,
                        _ => BacktraceMode::Disabled,
                    }
                }
            }

            #[derive(Debug)]
            struct BacktraceGuard {
                previous_rust_backtrace_value: BacktraceMode,
                test_name:                     String,
                enable_backtrace:              bool,
            }

            impl BacktraceGuard {
                /// Constructor for BacktraceGuard that sets
                /// the previous backtrace state, then (if `enable_backtrace`
                /// is false) updates RUST_BACKTRACE to "0" to disable it.
                ///
                /// This fixes the mismatch so that code can call:
                /// `BacktraceGuard::new("test_name".to_string(), <bool>)`.
                fn new(test_name: String, enable_backtrace: bool) -> Self {
                    tracing::debug!(
                        %test_name,
                        %enable_backtrace,
                        "Creating new BacktraceGuard"
                    );

                    let previous_rust_backtrace_value = BacktraceMode::from_env();
                    if !enable_backtrace {
                        tracing::info!(
                            "Disabling RUST_BACKTRACE for test: {}",
                            test_name
                        );
                        std::env::set_var("RUST_BACKTRACE", "0");
                    }

                    BacktraceGuard {
                        previous_rust_backtrace_value,
                        test_name,
                        enable_backtrace,
                    }
                }

                pub fn maybe_print_backtrace(&self) {
                    match self.previous_rust_backtrace_value {
                        BacktraceMode::Enabled | BacktraceMode::Full => {
                            self.print_backtrace();
                        }
                        BacktraceMode::Disabled => {}
                    }
                }

                pub fn print_backtrace(&self) {
                    let header = format!(
                        "===== Backtrace for test: {} =====",
                        self.test_name
                    );
                    let backtrace = format!(
                        "{:#?}",
                        std::backtrace::Backtrace::force_capture()
                    );

                    tracing::debug!("Printing backtrace header...");
                    println!("{}", header.bright_black());
                    println!("{}", backtrace.bright_black());
                }

                /// Restore the previous RUST_BACKTRACE value
                pub fn restore_previous_rust_backtrace_value(&self) {
                    match &self.previous_rust_backtrace_value {
                        BacktraceMode::Enabled => {
                            tracing::trace!(
                                "Restoring RUST_BACKTRACE to '1' after test: {}",
                                self.test_name
                            );
                            std::env::set_var("RUST_BACKTRACE", "1");
                        }
                        BacktraceMode::Full => {
                            tracing::trace!(
                                "Restoring RUST_BACKTRACE to 'full' after test: {}",
                                self.test_name
                            );
                            std::env::set_var("RUST_BACKTRACE", "full");
                        }
                        BacktraceMode::Disabled => {
                            tracing::trace!(
                                "RUST_BACKTRACE was disabled; resetting to '0' after test: {}",
                                self.test_name
                            );
                            std::env::set_var("RUST_BACKTRACE", "0");
                        }
                    }
                }
            }

            impl Drop for BacktraceGuard {
                fn drop(&mut self) {
                    if std::thread::panicking() {
                        if self.enable_backtrace {
                            tracing::debug!(
                                "Panic detected; printing backtrace for test: {}",
                                self.test_name
                            );
                            self.maybe_print_backtrace();
                        } else {
                            tracing::debug!(
                                "Panic detected, but `enable_backtrace` is false; skipping backtrace."
                            );
                        }
                    }
                    self.restore_previous_rust_backtrace_value();
                }
            }
        }
    }
}
