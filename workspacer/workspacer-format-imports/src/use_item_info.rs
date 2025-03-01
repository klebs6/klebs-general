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
}
