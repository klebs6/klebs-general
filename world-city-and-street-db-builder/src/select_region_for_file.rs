crate::ix!();

/// Attempts to determine which region a given file belongs to. Returns `Some` if found,
/// or `None` if the file does not match any known region heuristics.
pub fn select_region_for_file(
    file_path: &Path,
    known_regions: &[WorldRegion],
    base_dir: &Path,
) -> Option<WorldRegion> {
    trace!(
        "select_region_for_file: file_path={:?}, base_dir={:?}",
        file_path,
        base_dir
    );
    find_region_for_file(file_path, known_regions, base_dir)
}
