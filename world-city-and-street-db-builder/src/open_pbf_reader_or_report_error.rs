// ---------------- [ File: src/open_pbf_reader_or_report_error.rs ]
crate::ix!();

/// Helper that attempts to open the OSM PBF file. If successful, returns the reader.
/// On failure, sends the error through `tx` and returns `None`.
pub fn open_pbf_reader_or_report_error(
    path: &PathBuf,
    tx: &std::sync::mpsc::SyncSender<Result<WorldAddress, OsmPbfParseError>>,
) -> Option<osmpbf::ElementReader<std::io::BufReader<std::fs::File>>> {
    trace!("open_pbf_reader_or_report_error: Opening OSM PBF at {:?}", path);

    match open_osm_pbf_reader(path) {
        Ok(reader) => {
            debug!("open_pbf_reader_or_report_error: Successfully opened {:?}", path);
            Some(reader)
        }
        Err(e) => {
            error!("open_pbf_reader_or_report_error: Failed to open {:?}: {:?}", path, e);
            let _ = tx.send(Err(e));
            None
        }
    }
}
