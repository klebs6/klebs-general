// ---------------- [ File: workspacer-consolidate/src/gather_crate_items.rs ]
crate::ix!();

pub fn gather_crate_items(
    sf: &ra_ap_syntax::SourceFile,
    options: &ConsolidationOptions,
    file_path: &PathBuf,
    crate_path: &PathBuf,
) -> Vec<ConsolidatedItem> {
    gather_items_in_node(sf.syntax(), options, file_path, crate_path)
}
