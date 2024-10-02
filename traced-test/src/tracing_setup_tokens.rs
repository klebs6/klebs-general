crate::ix!();

impl TracedTestGenerator {

    /// Generates the tracing setup tokens.
    pub fn tracing_setup_tokens(&self) -> TokenStream2 {

        let test_name                = self.name().to_string();
        let should_trace_on_success  = self.should_trace_on_success();
        let should_trace_on_failure  = self.should_trace_on_failure();
        let def_backtrace_guard      = self.define_backtrace_guard();
        let def_flush_logs_if_needed = self.define_flush_logs_if_needed();
        let def_tracing_guard        = self.define_tracing_guard();
        let def_end_of_test_guard    = self.define_end_of_test_guard();
        let def_should_trace_trait   = self.define_should_trace_trait();

        quote! {
            use colored::Colorize;
            #def_should_trace_trait
            #def_backtrace_guard
            #def_flush_logs_if_needed
            #def_tracing_guard
            #def_end_of_test_guard

            println!("{}", format!("===== BEGIN_TEST: {} =====", #test_name).blue().bold());

            let local_subscriber = setup_buffered_tracing(Some(#test_name));
            let _guard           = tracing::subscriber::set_default(local_subscriber.clone());
            let span             = tracing::span!(tracing::Level::INFO, "test_trace", test_name = #test_name);
            let _enter           = span.enter();

            let test_failed = ::std::sync::Arc::new(::std::sync::Mutex::new(false));
            let test_failed_clone = test_failed.clone();

            let logs_already_flushed = ::std::sync::Arc::new(::std::sync::Mutex::new(false));

            let _end_of_test_guard = EndOfTestGuard;

            // Set RUST_BACKTRACE to "0" to suppress default panic backtrace
            let _backtrace_guard = {
                let previous_value = std::env::var("RUST_BACKTRACE").ok();
                // Set RUST_BACKTRACE to "0" to suppress default backtrace
                std::env::set_var("RUST_BACKTRACE", "0");

                BacktraceGuard { previous_value, test_name: #test_name.to_string() }
            };

            // Instantiate TracingGuard to ensure logs are flushed conditionally
            let _tracing_guard = TracingGuard {
                local_subscriber: local_subscriber.clone(),
                test_failed: test_failed.clone(),
                should_trace_on_success: #should_trace_on_success,
                should_trace_on_failure: #should_trace_on_failure,
                logs_already_flushed: logs_already_flushed.clone(),
            };
        }
    }
}
