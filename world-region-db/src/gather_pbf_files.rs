// ---------------- [ File: src/gather_pbf_files.rs ]
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

#[cfg(test)]
mod gather_pbf_files_tests {
    use super::*;

    /// Helper to create a `.pbf` file at the specified path with some bytes.
    async fn create_fake_pbf_file(path: &std::path::Path) {
        let mut file = File::create(path).await.expect("failed to create file");
        // Write minimal or even empty data, just so the file exists
        file.write_all(b"fake pbf data");
    }

    #[traced_test]
    fn test_gather_pbf_files_nonexistent_dir() {
        // Provide a directory path that doesn’t exist
        let missing = std::path::Path::new("/this/path/does/not/exist/hopefully");
        let result = gather_pbf_files(missing);
        assert!(result.is_err(), "Expected error for missing directory");
        match result.err().unwrap() {
            OsmPbfParseError::IoError(e) => {
                // Good: reading a nonexistent directory => IoError
                assert!(
                    e.kind() == std::io::ErrorKind::NotFound 
                     || e.kind() == std::io::ErrorKind::PermissionDenied,
                    "Expected NotFound or PermissionDenied, got: {:?}",
                    e
                );
            }
            other => panic!("Expected IoError, got: {:?}", other),
        }
    }

    #[traced_test]
    fn test_gather_pbf_files_empty_dir() {
        let tmp = TempDir::new().unwrap();
        let result = gather_pbf_files(tmp.path());
        assert!(result.is_ok(), "Empty directory => Ok");
        let pbf_files = result.unwrap();
        assert!(pbf_files.is_empty(), "No .pbf files => empty Vec");
    }

    #[traced_test]
    async fn test_gather_pbf_files_mixed_extensions() {
        let tmp = TempDir::new().unwrap();
        let dir_path = tmp.path();

        // Create some files
        // a) "one.pbf" => valid
        // b) "two.txt" => not .pbf
        // c) "three.PBF" => uppercase extension => not recognized
        {
            create_fake_pbf_file(&dir_path.join("one.pbf")).await;
            // text file
            let _ = File::create(dir_path.join("two.txt")).await.unwrap();
            // uppercase extension
            create_fake_pbf_file(&dir_path.join("three.PBF")).await;
        }

        let result = gather_pbf_files(dir_path);
        assert!(result.is_ok());
        let pbf_files = result.unwrap();

        // We only expect 1: "one.pbf"
        assert_eq!(pbf_files.len(), 1, "Only the strictly-lowercase .pbf extension is matched");
        let only_path = &pbf_files[0];
        assert_eq!(only_path.file_name().unwrap(), "one.pbf");
    }

    #[traced_test]
    async fn test_gather_pbf_files_multiple_pbf() {
        let tmp = TempDir::new().unwrap();
        let dir_path = tmp.path();

        // Create multiple .pbf files
        create_fake_pbf_file(&dir_path.join("alpha.pbf")).await;
        create_fake_pbf_file(&dir_path.join("beta.pbf")).await;
        create_fake_pbf_file(&dir_path.join("gamma.pbf")).await;
        // Also a subdirectory with a .pbf inside => not returned
        let subdir = dir_path.join("subdir");
        std::fs::create_dir(&subdir).unwrap();
        create_fake_pbf_file(&subdir.join("delta.pbf")).await;

        let result = gather_pbf_files(dir_path).expect("should succeed listing directory");
        // Expect 3 from the top-level; "delta.pbf" is in subdir => not in the immediate read_dir
        assert_eq!(result.len(), 3, "Should see exactly the top-level .pbf files");
        // Sort them for stable checking
        let mut names: Vec<String> = result
            .iter()
            .map(|pb| pb.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        names.sort();
        assert_eq!(names, vec!["alpha.pbf", "beta.pbf", "gamma.pbf"]);
    }

    #[traced_test]
    async fn test_gather_pbf_files_error_on_entry() {
        // On many OSes, if reading an individual entry fails (e.g. a special file),
        // the code returns an error. We'll demonstrate a partial approach:
        // We'll set up a scenario where there's a symlink or a file that we can't stat.
        // This might be OS-specific. We'll do a partial test that we can at least see an error.
        // 
        // If you can't or don't want to replicate an error reading a single entry, skip this scenario.
        // We'll do a partial approach for demonstration.

        let tmp = TempDir::new().unwrap();
        let dir_path = tmp.path();
        create_fake_pbf_file(&dir_path.join("ok.pbf")).await;

        // We won't attempt advanced manipulation here. 
        // If you want to forcibly cause an error in read_dir or while reading an entry,
        // you might set the directory to be unreadable or create a broken symlink:
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            // Create a symlink that points to nowhere
            let _ = symlink("/non/existent/path", dir_path.join("broken_symlink"));
        }

        let result = gather_pbf_files(dir_path);
        // Because the iteration might error out, we might see an error or might not (on some systems).
        // We'll do a partial check:
        match result {
            Ok(pbf_files) => {
                // Possibly everything is fine or the OS doesn't fail on a broken symlink => 
                // we see "ok.pbf" in the list
                assert_eq!(pbf_files.len(), 1, "Should see one .pbf if the symlink didn't hamper iteration");
            }
            Err(e) => {
                // If the system balks at the broken symlink => we get an IoError
                match e {
                    OsmPbfParseError::IoError(_io) => {
                        // acceptable
                    }
                    other => panic!("Unexpected error variant: {:?}", other),
                }
            }
        }
    }
}
