// ---------------- [ File: src/use_item_info.rs ]
crate::ix!();

/// Private struct => changed to public to avoid E0616
#[derive(Getters,Debug,Builder,Clone)]
#[builder(setter(into))]
#[getset(get="pub")]
pub struct UseItemInfo {
    leading_comments: Vec<String>,
    raw_text:         String,
    range_start:      usize,
    range_end:        usize,
    visibility:       String,
    path_list:        String,

    // ADDED: store any trailing comment on the same line, e.g. "// trailing comment"
    trailing_comment: Option<String>,
}
