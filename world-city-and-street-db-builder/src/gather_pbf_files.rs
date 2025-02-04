// ---------------- [ File: src/gather_pbf_files.rs ]
crate::ix!();

/// Reads the specified directory, returning a `Vec<PathBuf>` of all `.pbf` files found.
///
/// # Returns
///
/// * `Ok(Vec<PathBuf>)` if the directory is accessible.
/// * `Err(OsmPbfParseError)` if reading the directory fails.
pub fn gather_pbf_files(pbf_dir: &Path) -> Result<Vec<PathBuf>, OsmPbfParseError> {
    trace!("gather_pbf_files: scanning directory {:?}", pbf_dir);
    let entries = std::fs::read_dir(pbf_dir)
        .map_err(|io_err| OsmPbfParseError::IoError(io_err))?;

    let mut pbf_files = Vec::new();
    for entry_result in entries {
        let entry = match entry_result {
            Ok(e) => e,
            Err(e) => {
                error!("gather_pbf_files: error reading entry in {:?}: {}", pbf_dir, e);
                return Err(OsmPbfParseError::IoError(e));
            }
        };
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("pbf") {
            debug!("gather_pbf_files: found PBF file {:?}", path);
            pbf_files.push(path);
        }
    }

    Ok(pbf_files)
}
