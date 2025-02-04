// ---------------- [ File: src/validate_pbf_filename.rs ]
crate::ix!();

/// Validates that `pbf_path` has a filename matching what we'd expect for `region`.
/// It also checks for optional MD5 insertion in the filename.
/// Returns an error if mismatched or if filename is invalid/unreadable.
pub fn validate_pbf_filename(
    region:   &WorldRegion,
    pbf_path: &Path,
) -> Result<(), OsmPbfParseError> {
    // Attempt to extract the actual filename
    let actual_filename = pbf_path
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or_else(|| OsmPbfParseError::InvalidInputFile {
            reason: format!("Invalid filename: {:?}", pbf_path),
        })?;

    // Generate the “expected” filename. We pass `dir="."` to indicate “no path prefix,”
    // then remove that prefix from the result.
    let expected_path = expected_filename_for_region(Path::new("."), region);
    let expected_filename_str = expected_path.to_str().unwrap_or_default();

    // If these do not match (case-insensitive, optional MD5 block), return error
    if filenames_match(expected_filename_str, actual_filename) {
        Ok(())
    } else {
        Err(OsmPbfParseError::InvalidInputFile {
            reason: format!(
                "Provided PBF file '{:?}' does not match expected filename '{:?}' for region {:?}",
                actual_filename, 
                expected_filename_str, 
                region
            ),
        })
    }
}
