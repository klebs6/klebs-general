// ---------------- [ File: src/consolidation_options.rs ]
crate::ix!();

#[derive(Debug,Getters)]
#[getset(get="pub")]
pub struct ConsolidationOptions {
    include_docs:               bool,
    include_private:            bool,
    include_test_items:         bool,
    include_fn_bodies:          bool,
    include_fn_bodies_in_tests: bool,
    only_test_items:            bool,
}

impl ConsolidationOptions {

    /// Construct with default "off" for all toggles.
    pub fn new() -> Self {
        let opts = Self {
            include_docs:               false,
            include_private:            false,
            include_test_items:         false,
            include_fn_bodies:          false,
            include_fn_bodies_in_tests: false,
            only_test_items:            false,
        };
        opts.validate();
        opts
    }

    /// Enable doc comment extraction
    pub fn with_docs(mut self) -> Self {
        self.include_docs = true;
        self
    }

    /// Include private items (fn, struct, etc.).
    pub fn with_private_items(mut self) -> Self {
        self.include_private = true;
        self
    }

    /// Include items that appear in test contexts (e.g. `#[cfg(test)]`).
    pub fn with_test_items(mut self) -> Self {
        self.include_test_items = true;
        self
    }

    /// Include the (naive) function bodies as source text (for non-test items).
    pub fn with_fn_bodies(mut self) -> Self {
        self.include_fn_bodies = true;
        self
    }

    /// Include function bodies in test items. This is separate from `with_fn_bodies`.
    pub fn with_fn_bodies_in_tests(mut self) -> Self {
        self.include_fn_bodies_in_tests = true;
        self
    }

    /// Only display test items (skip all non-test items).
    /// This automatically sets `include_test_items = true`.
    pub fn with_only_test_items(mut self) -> Self {
        // If only_test_items is requested but include_test_items is off, we must turn it on.
        if !self.include_test_items {
            warn!("`only_test_items=true` requires `include_test_items=true`. Forcing it on.");
            self.include_test_items = true;
        }
        self.only_test_items = true;
        self
    }

    /// Optional: A small validation pass that logs warnings if flags are contradictory.
    /// You can call this after you've built the options (e.g. in a builder pattern),
    /// or you can embed these checks inline in each setter if you prefer.
    pub fn validate(&self) {
        // Example: If only_test_items=true, but also include_private=true, maybe warn:
        if self.only_test_items && self.include_private {
            warn!("You set `only_test_items=true` *and* `include_private=true`. Private non-test items won't appear anyway, so `include_private` has no effect in only-test mode.");
        }
        // Similarly, if only_test_items=true but with_fn_bodies=true:
        if self.only_test_items && self.include_fn_bodies {
            warn!("You set `only_test_items=true` and `include_fn_bodies=true`. Bodies for non-test items won't show up, so that combination might be redundant unless you also have test items with bodies enabled.");
        }
        // And so on for any other combos you consider confusing...
    }
}

#[cfg(test)]
#[disable]
mod test_consolidation_options {
    use super::*;
    use log::{LevelFilter, Record};
    use std::sync::Mutex;

    // Optional: a simple global log collector to capture warnings from `validate()`.
    // If you don’t care about verifying log output, you can remove this entire mechanism.
    static LOG_COLLECTOR: Mutex<Vec<String>> = Mutex::new(Vec::new());

    // A simple logger that pushes warnings into LOG_COLLECTOR.
    struct TestLogger;

    impl log::Log for TestLogger {
        fn enabled(&self, metadata: &log::Metadata) -> bool {
            metadata.level() <= log::Level::Warn
        }

        fn log(&self, record: &Record) {
            if self.enabled(record.metadata()) {
                let mut logs = LOG_COLLECTOR.lock().unwrap();
                logs.push(format!("{}", record.args()));
            }
        }

        fn flush(&self) {}
    }

    // We'll set up the logger once for all tests in this module.
    // If you prefer each test to do so individually, adapt accordingly.
    #[ctor::ctor]
    fn init_logger() {
        // Only set logger once globally. The .set_logger call can fail if repeated.
        static LOGGER: TestLogger = TestLogger;
        let _ = log::set_logger(&LOGGER);
        let _ = log::set_max_level(LevelFilter::Warn);
    }

    /// A helper to clear the global log collector between tests, if we want to test log messages.
    fn clear_logs() {
        let mut logs = LOG_COLLECTOR.lock().unwrap();
        logs.clear();
    }

    /// A helper to read out any warnings emitted since the last clear.
    fn get_logs() -> Vec<String> {
        LOG_COLLECTOR.lock().unwrap().clone()
    }

    // ------------------------------------------------------------------------
    // Test Cases
    // ------------------------------------------------------------------------

    /// 1) By default, all toggles are off (false).
    #[test]
    fn test_default_new() {
        clear_logs();
        let opts = ConsolidationOptions::new();
        // Validate is called inside new(), so if there's any contradictory default, we'd see a warning
        let logs = get_logs();
        assert!(logs.is_empty(), "No warnings expected for default constructor");
        // Now check each field:
        assert_eq!(opts.include_docs(), false);
        assert_eq!(opts.include_private(), false);
        assert_eq!(opts.include_test_items(), false);
        assert_eq!(opts.include_fn_bodies(), false);
        assert_eq!(opts.include_fn_bodies_in_tests(), false);
        assert_eq!(opts.only_test_items(), false);
    }

    /// 2) with_docs() => sets include_docs=true, others remain default.
    #[test]
    fn test_with_docs() {
        let opts = ConsolidationOptions::new().with_docs();
        assert_eq!(opts.include_docs(), true, "Docs should be on");
        assert_eq!(opts.include_private(), false);
        assert_eq!(opts.include_test_items(), false);
        assert_eq!(opts.include_fn_bodies(), false);
        assert_eq!(opts.include_fn_bodies_in_tests(), false);
        assert_eq!(opts.only_test_items(), false);
    }

    /// 3) with_private_items() => sets include_private=true.
    #[test]
    fn test_with_private_items() {
        let opts = ConsolidationOptions::new().with_private_items();
        assert_eq!(opts.include_docs(), false);
        assert_eq!(opts.include_private(), true, "private items on");
        assert_eq!(opts.include_test_items(), false);
        assert_eq!(opts.include_fn_bodies(), false);
        assert_eq!(opts.include_fn_bodies_in_tests(), false);
        assert_eq!(opts.only_test_items(), false);
    }

    /// 4) with_test_items() => sets include_test_items=true.
    #[test]
    fn test_with_test_items() {
        let opts = ConsolidationOptions::new().with_test_items();
        assert_eq!(opts.include_docs(), false);
        assert_eq!(opts.include_private(), false);
        assert_eq!(opts.include_test_items(), true, "test items on");
        assert_eq!(opts.include_fn_bodies(), false);
        assert_eq!(opts.include_fn_bodies_in_tests(), false);
        assert_eq!(opts.only_test_items(), false);
    }

    /// 5) with_fn_bodies() => sets include_fn_bodies=true (for non-test items).
    #[test]
    fn test_with_fn_bodies() {
        let opts = ConsolidationOptions::new().with_fn_bodies();
        assert!(opts.include_fn_bodies(), "fn bodies on for non-test");
        assert!(!opts.include_fn_bodies_in_tests());
    }

    /// 6) with_fn_bodies_in_tests() => sets include_fn_bodies_in_tests=true,
    ///    separate from with_fn_bodies (which handles non-test).
    #[test]
    fn test_with_fn_bodies_in_tests() {
        let opts = ConsolidationOptions::new().with_fn_bodies_in_tests();
        assert_eq!(opts.include_docs(), false);
        assert_eq!(opts.include_private(), false);
        assert_eq!(opts.include_test_items(), false);
        assert_eq!(opts.include_fn_bodies(), false);
        assert_eq!(opts.include_fn_bodies_in_tests(), true, "fn bodies on for test items");
        assert_eq!(opts.only_test_items(), false);
    }

    /// 7) with_only_test_items() => sets only_test_items=true, also forces include_test_items=true.
    ///    Should produce a log warning if include_test_items was previously false. 
    #[test]
    fn test_with_only_test_items_sets_include_test_items() {
        clear_logs();
        let opts = ConsolidationOptions::new().with_only_test_items();
        // Now only_test_items = true, include_test_items = true
        assert_eq!(opts.only_test_items(), true);
        assert_eq!(opts.include_test_items(), true, "Forces test items on");
        // Check if we got a warning from the code "requires `include_test_items=true`. Forcing it on."
        let logs = get_logs();
        assert_eq!(logs.len(), 1, "Should have exactly one log line about forcing test_items on");
        assert!(logs[0].contains("Forcing it on"), "Warn text must mention forcing test_items on");
    }

    /// 8) If we already had include_test_items=true, calling with_only_test_items() should not produce a new warning.
    #[test]
    fn test_with_only_test_items_already_including_test_items() {
        clear_logs();
        let opts = ConsolidationOptions::new()
            .with_test_items()
            .with_only_test_items();
        assert!(opts.include_test_items(), "We had it on already");
        assert!(opts.only_test_items(), "Still on");
        let logs = get_logs();
        assert!(logs.is_empty(), "No new warning if test_items was already on");
    }

    /// 9) If we call multiple chain calls, they accumulate. We'll do a chain that sets everything on.
    #[test]
    fn test_chaining_all_options() {
        clear_logs();
        let opts = ConsolidationOptions::new()
            .with_docs()
            .with_private_items()
            .with_test_items()
            .with_fn_bodies()
            .with_fn_bodies_in_tests()
            .with_only_test_items();
        // Checking final state
        assert!(opts.include_docs());
        assert!(opts.include_private());
        // Because we used with_test_items, it is true:
        assert!(opts.include_test_items());
        // Because we used with_fn_bodies:
        assert!(opts.include_fn_bodies());
        // Because we used with_fn_bodies_in_tests:
        assert!(opts.include_fn_bodies_in_tests());
        // because we used with_only_test_items => only_test_items = true
        assert!(opts.only_test_items());
        // Also that implies we forced include_test_items on if it wasn't.
        // We'll see if we get a log line about it.
        let logs = get_logs();
        // Possibly we only see 1 line about forcing it on if it wasn't already on
        // But we had with_test_items prior, so likely no warning about forcing.
        // depends on your code's order. If `with_only_test_items()` was last, it
        // won't produce a forced-on warning because test items was already set.
        // So we might expect logs to be empty.
        // Let's just check that no contradictory logs were produced.
        assert!(logs.is_empty(), "No contradictory logs expected with this chain");
    }

    /// 10) verify the internal `validate()` logic warns if we do contradictory combos, e.g. only_test_items + include_private => we see a warning.
    #[test]
    fn test_validate_warns_on_contradictory_combos() {
        clear_logs();
        // We'll do only_test_items + include_private
        let mut opts = ConsolidationOptions {
            include_docs: false,
            include_private: true,
            include_test_items: false,
            include_fn_bodies: false,
            include_fn_bodies_in_tests: false,
            only_test_items: true,
        };
        // Now call validate
        opts.validate();
        let logs = get_logs();
        assert!(!logs.is_empty(), "We expect at least one warning about contradictory combos");
        let found_private_warning = logs.iter().any(|line| line.contains("only_test_items=true") && line.contains("include_private=true"));
        assert!(found_private_warning, "We should see a warning about private + only_test_items");
    }
}
