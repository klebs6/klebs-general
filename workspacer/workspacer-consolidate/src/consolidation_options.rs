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
