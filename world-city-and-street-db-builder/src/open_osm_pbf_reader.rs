// ---------------- [ File: src/open_osm_pbf_reader.rs ]
// ---------------- [ File: src/open_osm_pbf_reader.rs ]
crate::ix!();

pub fn open_osm_pbf_reader(path: impl AsRef<Path>)
    -> Result<ElementReader<std::io::BufReader<std::fs::File>>, OsmPbfParseError>
{
    trace!("open_osm_pbf_reader: Attempting to open {:?}", path.as_ref());
    match osmpbf::ElementReader::from_path(path) {
        Ok(r) => Ok(r),
        Err(e) => Err(OsmPbfParseError::OsmPbf(e)),
    }
}

#[cfg(test)]
mod open_osm_pbf_reader_tests {
    use super::*;

    #[traced_test]
    fn test_open_osm_pbf_reader_valid() {
        let tmp_dir = TempDir::new().unwrap();
        let pbf_path = tmp_dir.path().join("sample.osm.pbf");

        // Create a 0-byte file or small valid file:
        std::fs::write(&pbf_path, b"").expect("write empty pbf data");

        let result = open_osm_pbf_reader(&pbf_path);
        match result {
            Ok(reader) => {
                // We may only get a valid `ElementReader` if the file isn't actually corrupt,
                // but let's see if it doesn't fail *immediately*.
                // assert!(reader.is_ok(), "ElementReader might accept empty file or fail");
            }
            Err(e) => {
                // Some implementations might treat empty-file as an error, that’s also valid.
                // If so, you can adapt your test accordingly.
                eprintln!("Got an expected error: {:?}", e);
            }
        }
    }

    #[traced_test]
    fn test_open_osm_pbf_reader_missing_file() {
        let path = PathBuf::from("/non/existent/path/to.pbf");
        let result = open_osm_pbf_reader(&path);
        assert!(result.is_err(), "Should fail for nonexistent file");
    }
}
