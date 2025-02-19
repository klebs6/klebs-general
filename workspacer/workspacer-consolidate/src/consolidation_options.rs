crate::ix!();

/// A small "options" struct controlling whether to include doc comments, private items, etc.
/// We do NOT expose its fields publicly; we use builder-like methods for configuration.
#[derive(Debug,Getters)]
#[getset(get="pub")]
pub struct ConsolidationOptions {
    include_docs:       bool,
    include_private:    bool,
    include_test_items: bool,
    include_fn_bodies:  bool,
}

impl ConsolidationOptions {

    /// Construct with default "off" for all toggles.
    pub fn new() -> Self {
        Self {
            include_docs:       false,
            include_private:    true,
            include_test_items: false,
            include_fn_bodies:  false,
        }
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

    /// Include the (naive) function bodies as source text.
    pub fn with_fn_bodies(mut self) -> Self {
        self.include_fn_bodies = true;
        self
    }
}
