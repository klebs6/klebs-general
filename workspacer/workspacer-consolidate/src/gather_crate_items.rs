crate::ix!();

pub fn gather_crate_items(
    sf: &ra_ap_syntax::SourceFile,
    options: &ConsolidationOptions,
) -> Vec<ConsolidatedItem> {
    gather_items_in_node(sf.syntax(), options)
}


