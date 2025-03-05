// ---------------- [ File: workspacer-format-imports/src/gather_comments_state.rs ]
crate::ix!();

/// Tracks what we've collected and where we are in the scanning process.
#[derive(Setters,MutGetters,Getters,Debug)]
#[getset(get="pub",set="pub",get_mut="pub")]
pub struct GatherCommentsState {
    /// The collected comment lines in top-to-bottom order.
    collected: Vec<String>,
    /// Whether we've already found a comment during the climb.
    found_comment: bool,
    /// How many newlines have appeared since the last non-whitespace item.
    whitespace_newlines_since_last_nonws: usize,
}

impl GatherCommentsState {
    pub fn new() -> Self {
        Self {
            collected: Vec::new(),
            found_comment: false,
            whitespace_newlines_since_last_nonws: 0,
        }
    }
}
