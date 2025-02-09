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

#[cfg(test)]
mod test_open_pbf_reader_or_report_error {
    use super::*;
    use std::sync::mpsc;
    use tempfile::TempDir;
    use std::fs::{File, OpenOptions};
    use std::io::Write;

    /// Tests the success path where `open_osm_pbf_reader` can open a valid file.
    /// We'll create a tiny file, possibly not a real .osm.pbf, but enough to test the file I/O part.
    /// The parsing might fail later, but we're only testing the open step here.
    #[test]
    fn test_opens_valid_file_returns_some() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test_data.pbf");

        // Create an empty file (not necessarily a valid .osm.pbf),
        // but good enough to test the open behavior.
        File::create(&file_path).expect("Failed to create test file");

        // Set up the channel
        let (tx, rx) = mpsc::sync_channel::<Result<WorldAddress, OsmPbfParseError>>(10);

        // Call the function
        let result = open_pbf_reader_or_report_error(&file_path, &tx);

        // Verify it returns Some(...)
        assert!(result.is_some(), "Expected Some(reader) for an existing file");
        // The channel should remain empty because we didn't fail
        assert!(rx.try_recv().is_err(), "No error should be sent over channel");
    }

    /// Tests the failure path when the file does not exist, expecting None and an error to be sent.
    #[test]
    fn test_file_not_found_returns_none_sends_error() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let non_existent_path = temp_dir.path().join("no_such_file.pbf");

        let (tx, rx) = mpsc::sync_channel::<Result<WorldAddress, OsmPbfParseError>>(10);

        let result = open_pbf_reader_or_report_error(&non_existent_path, &tx);
        assert!(result.is_none(), "Should return None for a non-existent file");

        // The channel should have exactly one error
        let err_msg = rx.try_recv()
            .expect("Expected an error to be sent")
            .unwrap_err();
        match err_msg {
            OsmPbfParseError::OsmPbf(e) => {
                // Rocks or Pbf parse error for a missing file; check the partial wording
                assert!(
                    e.to_string().contains("No such file") ||
                    e.to_string().contains("not found") ||
                    e.to_string().contains("IO error"),
                    "Error message should indicate missing file. Got: {:?}", e
                );
            },
            other => panic!("Expected OsmPbf(...) error, got {:?}", other),
        }
        // No more messages in the channel
        assert!(rx.try_recv().is_err(), "No further messages expected");
    }

    /// Tests the failure path when the path is a directory rather than a file.
    /// We expect None and an error to be sent.
    #[test]
    fn test_open_directory_returns_none_sends_error() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        // The `temp_dir` itself is a directory
        let dir_path = temp_dir.path().to_path_buf();

        let (tx, rx) = mpsc::sync_channel::<Result<WorldAddress, OsmPbfParseError>>(10);

        let result = open_pbf_reader_or_report_error(&dir_path, &tx);
        assert!(result.is_none(), "Should return None if the path is a directory");

        // The channel should receive an error
        let err_msg = rx.try_recv()
            .expect("Expected an error to be sent")
            .unwrap_err();
        match err_msg {
            OsmPbfParseError::OsmPbf(e) => {
                // For a directory, likely a "Is a directory" or "IO error" message
                assert!(
                    e.to_string().contains("directory") ||
                    e.to_string().contains("Is a directory"),
                    "Error message should reflect that it's not a valid file. Got: {:?}", e
                );
            },
            other => panic!("Expected OsmPbf(...) error, got {:?}", other),
        }
        // No more messages
        assert!(rx.try_recv().is_err(), "No further messages expected");
    }

    /// Optionally, if you want to test a locked file or permission error,
    /// you could do so under UNIX. This is more advanced and not always portable.
    #[test]
    #[cfg(unix)]
    fn test_permission_denied_sends_error() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("locked_file.pbf");

        // Create file, then remove all permissions
        File::create(&file_path).expect("Failed to create file");
        let mut perms = std::fs::metadata(&file_path).unwrap().permissions();
        perms.set_mode(0o000); // no read, no write
        std::fs::set_permissions(&file_path, perms).unwrap();

        let (tx, rx) = mpsc::sync_channel::<Result<WorldAddress, OsmPbfParseError>>(10);

        let result = open_pbf_reader_or_report_error(&file_path, &tx);
        assert!(result.is_none(), "Should return None if the file is unreadable");

        // The channel should receive an error
        let err_msg = rx.try_recv()
            .expect("Expected an error to be sent")
            .unwrap_err();
        match err_msg {
            OsmPbfParseError::OsmPbf(e) => {
                // Typically "Permission denied"
                assert!(
                    e.to_string().contains("Permission denied"),
                    "Expected permission error. Got: {:?}", e
                );
            },
            other => panic!("Expected OsmPbf(...) error, got {:?}", other),
        }
    }

    /// Tests that if the function returns Some(reader), no error is sent in the channel.
    /// Already covered in the success test, but let's confirm no partial failures occur.
    #[test]
    fn test_no_error_sent_on_success() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("valid_file.pbf");

        // Creating a 1-byte file
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&file_path)
            .expect("Failed to create file");
        writeln!(file, "O").expect("Write 1 byte to file");

        let (tx, rx) = mpsc::sync_channel::<Result<WorldAddress, OsmPbfParseError>>(10);

        let result = open_pbf_reader_or_report_error(&file_path, &tx);
        assert!(result.is_some());
        // Channel should remain empty
        assert!(rx.try_recv().is_err(), "No messages should be present");
    }
}
