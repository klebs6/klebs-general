// ---------------- [ File: src/repl_autocomplete_mode.rs ]
crate::ix!();

/// (1) We define an “autocomplete mode” so the user can switch between City vs. Street fuzzy completions.
#[derive(Debug,Clone)]
pub enum AutocompleteMode {
    City,
    Street,
}
