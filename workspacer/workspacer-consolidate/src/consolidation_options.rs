// ---------------- [ File: src/consolidation_options.rs ]
crate::ix!();

/// A collection of user-defined toggles controlling how items are gathered and displayed.
/// In older versions of your code, you may have used `log` or `ctor`. This version removes
/// those external dependencies, using simple `eprintln!` calls for warnings instead.
#[derive(Debug, Getters)]
#[getset(get = "pub")]
pub struct ConsolidationOptions {
    include_docs:               bool,
    include_private:            bool,
    include_test_items:         bool,
    include_fn_bodies:          bool,
    include_fn_bodies_in_tests: bool,
    only_test_items:            bool,
}

impl ConsolidationOptions {
    /// Construct with all toggles turned off by default.
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

    /// Turn on doc comment extraction (`include_docs = true`).
    pub fn with_docs(mut self) -> Self {
        self.include_docs = true;
        self
    }

    /// Include private items (e.g. non-`pub fn`, private structs, etc.).
    pub fn with_private_items(mut self) -> Self {
        self.include_private = true;
        self
    }

    /// Include test items (marked `#[cfg(test)]`, etc.).
    pub fn with_test_items(mut self) -> Self {
        self.include_test_items = true;
        self
    }

    /// Include function bodies for normal (non-test) items.
    pub fn with_fn_bodies(mut self) -> Self {
        self.include_fn_bodies = true;
        self
    }

    /// Include function bodies for test items. 
    /// (Separate from `with_fn_bodies`, in case you want bodies for non-test only.)
    pub fn with_fn_bodies_in_tests(mut self) -> Self {
        self.include_fn_bodies_in_tests = true;
        self
    }

    /// Show *only* test items, skipping everything else.
    /// Automatically sets `include_test_items = true` if not already set.
    pub fn with_only_test_items(mut self) -> Self {
        if !self.include_test_items {
            eprintln!("[WARN] `only_test_items=true` requires `include_test_items=true`. Forcing it on.");
            self.include_test_items = true;
        }
        self.only_test_items = true;
        self
    }

    /// A small validation method that logs warnings via `eprintln!` if 
    /// certain combinations of flags are contradictory or possibly confusing.
    pub fn validate(&self) {
        // Example: if you want to warn that `only_test_items` + `include_private` is mostly redundant:
        if self.only_test_items && self.include_private {
            eprintln!("[WARN] You set `only_test_items=true` and `include_private=true`. \
                       Private non-test items won't appear anyway (they're not test items), so this might be redundant.");
        }

        // Another example: if `only_test_items` + `include_fn_bodies` might not do anything for non-test items:
        if self.only_test_items && self.include_fn_bodies && !self.include_fn_bodies_in_tests {
            eprintln!("[WARN] `only_test_items=true` and `include_fn_bodies=true`, but no `include_fn_bodies_in_tests`. \
                       You won't see non-test bodies. Possibly you meant `.with_fn_bodies_in_tests()`?");
        }
        // etc. Expand for any combos you want to warn about.
    }
}

#[cfg(test)]
mod test_consolidation_options {
    use super::*;

    /// 1) By default, all toggles are off.
    #[test]
    fn test_default_new() {
        let opts = ConsolidationOptions::new();
        assert_eq!(*opts.include_docs(), false);
        assert_eq!(*opts.include_private(), false);
        assert_eq!(*opts.include_test_items(), false);
        assert_eq!(*opts.include_fn_bodies(), false);
        assert_eq!(*opts.include_fn_bodies_in_tests(), false);
        assert_eq!(*opts.only_test_items(), false);
    }

    /// 2) `with_docs()` => `include_docs = true`, everything else default.
    #[test]
    fn test_with_docs() {
        let opts = ConsolidationOptions::new().with_docs();
        assert_eq!(*opts.include_docs(), true);
        assert_eq!(*opts.include_private(), false);
        assert_eq!(*opts.include_test_items(), false);
        assert_eq!(*opts.include_fn_bodies(), false);
        assert_eq!(*opts.include_fn_bodies_in_tests(), false);
        assert_eq!(*opts.only_test_items(), false);
    }

    /// 3) `with_private_items()` => `include_private = true`.
    #[test]
    fn test_with_private_items() {
        let opts = ConsolidationOptions::new().with_private_items();
        assert!(opts.include_private());
        // others default
        assert!(!opts.include_docs());
        assert!(!opts.include_test_items());
        assert!(!opts.include_fn_bodies());
        assert!(!opts.include_fn_bodies_in_tests());
        assert!(!opts.only_test_items());
    }

    /// 4) `with_test_items()` => `include_test_items = true`.
    #[test]
    fn test_with_test_items() {
        let opts = ConsolidationOptions::new().with_test_items();
        assert!(opts.include_test_items());
    }

    /// 5) `with_fn_bodies()` => `include_fn_bodies = true`.
    #[test]
    fn test_with_fn_bodies() {
        let opts = ConsolidationOptions::new().with_fn_bodies();
        assert!(opts.include_fn_bodies());
        assert!(!opts.include_fn_bodies_in_tests());
    }

    /// 6) `with_fn_bodies_in_tests()` => `include_fn_bodies_in_tests = true`.
    #[test]
    fn test_with_fn_bodies_in_tests() {
        let opts = ConsolidationOptions::new().with_fn_bodies_in_tests();
        assert!(opts.include_fn_bodies_in_tests());
    }

    /// 7) `with_only_test_items()` => sets `only_test_items = true`, forces `include_test_items = true`.
    #[test]
    fn test_with_only_test_items_sets_include_test_items() {
        let opts = ConsolidationOptions::new().with_only_test_items();
        // Because we forced it on
        assert!(opts.only_test_items());
        assert!(opts.include_test_items());
    }

    /// 8) Already had `include_test_items = true`, calling `with_only_test_items()` won't produce extra warnings in this simplified version.
    #[test]
    fn test_with_only_test_items_already_including_test_items() {
        let opts = ConsolidationOptions::new().with_test_items().with_only_test_items();
        assert!(opts.only_test_items());
        assert!(opts.include_test_items());
    }

    /// 9) Chain multiple calls => everything on.
    #[test]
    fn test_chaining_all_options() {
        let opts = ConsolidationOptions::new()
            .with_docs()
            .with_private_items()
            .with_test_items()
            .with_fn_bodies()
            .with_fn_bodies_in_tests()
            .with_only_test_items();

        // verify final toggles
        assert!(opts.include_docs());
        assert!(opts.include_private());
        assert!(opts.include_test_items());
        assert!(opts.include_fn_bodies());
        assert!(opts.include_fn_bodies_in_tests());
        // plus forced on only_test_items
        assert!(opts.only_test_items());
    }
}
