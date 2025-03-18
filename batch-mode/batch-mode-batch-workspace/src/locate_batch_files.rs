// ---------------- [ File: src/locate_batch_files.rs ]
crate::ix!();

#[async_trait]
impl<T> LocateBatchFiles for T
where
    for<'async_trait> T: BatchWorkspaceInterface + Send + Sync + 'async_trait,
{
    type Error = BatchWorkspaceError;
    async fn locate_batch_files(
        self:  Arc<Self>,
        index: &BatchIndex
    ) -> Result<Option<BatchFileTriple>, Self::Error> {
        trace!("attempting to locate batch files for index: {:?}", index);

        // Get the regex pattern for the specified index to match filenames
        let file_pattern = index.file_pattern();

        let mut input               = None;
        let mut output              = None;
        let mut error               = None;
        let mut associated_metadata = None;

        let mut entries = fs::read_dir(self.workdir()).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            // Get filename as a &str
            let filename = match path.file_name().and_then(|name| name.to_str()) {
                Some(name) => name,
                None => {
                    trace!("skipping a file with non-UTF8 name: {:?}", path);
                    continue;
                }
            };

            // Use the precompiled regex pattern to match filenames
            let captures = match file_pattern.captures(filename) {
                Some(captures) => captures,
                None => {
                    debug!("filename does not match the expected pattern: {:?}", filename);
                    continue;
                }
            };

            // Extract the type of the file from the capture group
            let file_type = captures.get(1).map(|m| m.as_str());

            match file_type {
                Some("input") => {
                    if input.is_some() {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Multiple input files found"
                        ).into());
                    }
                    debug!("found input file: {:?}", path);
                    input = Some(path);
                }
                Some("output") => {
                    if output.is_some() {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Multiple output files found"
                        ).into());
                    }
                    debug!("found output file: {:?}", path);
                    output = Some(path);
                }
                Some("error") => {
                    if error.is_some() {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Multiple error files found"
                        ).into());
                    }
                    debug!("found error file: {:?}", path);
                    error = Some(path);
                }
                Some("metadata") => {
                    if associated_metadata.is_some() {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "Multiple associated_metadata files found"
                        ).into());
                    }
                    debug!("found associated_metadata file: {:?}", path);
                    associated_metadata = Some(path);
                }
                _ => {
                    trace!("skipping unrecognized file type: {:?}", filename);
                    continue;
                }
            }
        }

        if input.is_none() && output.is_none() && error.is_none() && associated_metadata.is_none() {
            debug!("no batch files found in directory for index {:?}: {:?}", index, self.workdir());
            Ok(None)
        } else {
            debug!(
                "batch files located for index {:?} - input: {:?}, output: {:?}, error: {:?}, metadata: {:?}",
                index, input, output, error, associated_metadata
            );
            Ok(Some(BatchFileTriple::new_direct(
                        index, 
                        input, 
                        output, 
                        error, 
                        associated_metadata, 
                        self.clone()
            )))
        }
    }
}

#[cfg(test)]
mod locate_batch_files_exhaustive_tests {
    use super::*;

    #[traced_test]
    async fn test_locate_batch_files_usize() -> Result<(),BatchWorkspaceError> {

        let workspace = BatchWorkspace::new_temp().await?;
        let workdir   = workspace.workdir();

        fs::write(workdir.join("batch_input_4.jsonl"), b"test").await?;
        fs::write(workdir.join("batch_output_4.jsonl"), b"test").await?;
        fs::write(workdir.join("batch_error_4.jsonl"), b"test").await?;

        let batch_files = workspace.clone().locate_batch_files(&BatchIndex::Usize(4)).await?.unwrap();
        assert_eq!(*batch_files.input(), Some(workdir.join("batch_input_4.jsonl")));
        assert_eq!(*batch_files.output(), Some(workdir.join("batch_output_4.jsonl")));
        assert_eq!(*batch_files.error(), Some(workdir.join("batch_error_4.jsonl")));

        Ok(())
    }

    #[traced_test]
    async fn test_locate_batch_files_uuid() -> Result<(),BatchWorkspaceError> {
        let workspace = BatchWorkspace::new_temp().await?;
        let workdir   = workspace.workdir();

        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        fs::write(workdir.join(format!("batch_input_{}.jsonl", uuid)), b"test").await?;
        fs::write(workdir.join(format!("batch_output_{}.jsonl", uuid)), b"test").await?;

        let batch_files = workspace.clone().locate_batch_files(&BatchIndex::from_uuid_str(uuid)?).await?.unwrap();
        assert_eq!(*batch_files.input(), Some(workdir.join(format!("batch_input_{}.jsonl", uuid))));
        assert_eq!(*batch_files.output(), Some(workdir.join(format!("batch_output_{}.jsonl", uuid))));
        assert_eq!(*batch_files.error(), None);

        Ok(())
    }

    #[traced_test]
    async fn test_locate_batch_files_no_files() -> Result<(),BatchWorkspaceError> {
        let workspace = BatchWorkspace::new_temp().await?;

        let batch_files = workspace.locate_batch_files(&BatchIndex::Usize(4)).await?;
        assert!(batch_files.is_none());

        Ok(())
    }

    #[traced_test]
    async fn test_locate_batch_files_ignores_invalid_files() -> Result<(),BatchWorkspaceError> {
        let workspace = BatchWorkspace::new_temp().await?;
        let workdir   = workspace.workdir();

        // Write one valid input file
        fs::write(workdir.join("batch_input_4.jsonl"), b"test").await?;
        // Write one file that doesn't match the pattern
        fs::write(workdir.join("batch_input_4_duplicate.jsonl"), b"test").await?;

        let result = workspace.clone().locate_batch_files(&BatchIndex::Usize(4)).await?;
        assert!(result.is_some(), "Expected to find the valid batch input file");

        let batch_files = result.unwrap();
        assert_eq!(*batch_files.input(), Some(workdir.join("batch_input_4.jsonl")));
        assert!(batch_files.output().is_none());
        assert!(batch_files.error().is_none());

        Ok(())
    }


    /// Ensures we can handle the scenario in which there are no matching files at all for the given index.
    #[traced_test]
    async fn returns_none_when_no_files_present_for_index() {
        info!("Starting test: returns_none_when_no_files_present_for_index");
        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
        let index = BatchIndex::Usize(9999);

        debug!("Invoking locate_batch_files with empty workspace and index=9999");
        let result = workspace.clone().locate_batch_files(&index).await;
        debug!("Result: {:?}", result);

        assert!(result.is_ok(), "Should not error out if no files found");
        let triple_option = result.unwrap();
        assert!(triple_option.is_none(), "No files => we expect None");
        info!("Finished test: returns_none_when_no_files_present_for_index");
    }

    /// Ensures we can locate a single input file with no other files present.
    #[traced_test]
    async fn locates_single_input_file() {
        info!("Starting test: locates_single_input_file");
        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
        let index = BatchIndex::Usize(10);
        let filename = format!("batch_input_{}.jsonl", 10);

        let path = workspace.workdir().join(&filename);
        let content = b"some content for input";
        fs::write(&path, content).await.expect("Failed to write input file");

        let result = workspace.clone().locate_batch_files(&index).await;
        debug!("Result: {:?}", result);
        assert!(result.is_ok(), "Locating single input file should succeed");

        let triple_option = result.unwrap();
        assert!(triple_option.is_some(), "Expected to find a triple with the input file");
        let triple = triple_option.unwrap();
        assert_eq!(*triple.index(), index, "Index should match");
        assert_eq!(*triple.input(), Some(path.clone()));
        assert!(triple.output().is_none(), "No output file");
        assert!(triple.error().is_none(), "No error file");
        assert!(triple.associated_metadata().is_none(), "No metadata file");

        info!("Finished test: locates_single_input_file");
    }

    /// Ensures we can locate a single output file with no other files present.
    #[traced_test]
    async fn locates_single_output_file() {
        info!("Starting test: locates_single_output_file");
        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
        let index = BatchIndex::Usize(11);
        let filename = format!("batch_output_{}.jsonl", 11);

        let path = workspace.workdir().join(&filename);
        let content = b"some output data";
        fs::write(&path, content).await.expect("Failed to write output file");

        let result = workspace.clone().locate_batch_files(&index).await;
        debug!("Result: {:?}", result);
        assert!(result.is_ok());

        let triple_option = result.unwrap();
        assert!(triple_option.is_some(), "Should find a triple with the output file only");
        let triple = triple_option.unwrap();
        assert_eq!(*triple.index(), index);
        assert!(triple.input().is_none());
        assert_eq!(*triple.output(), Some(path.clone()));
        assert!(triple.error().is_none());
        assert!(triple.associated_metadata().is_none());

        info!("Finished test: locates_single_output_file");
    }

    /// Ensures we can locate a single error file with no other files present.
    #[traced_test]
    async fn locates_single_error_file() {
        info!("Starting test: locates_single_error_file");
        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
        let index = BatchIndex::Usize(12);
        let filename = format!("batch_error_{}.jsonl", 12);

        let path = workspace.workdir().join(&filename);
        fs::write(&path, b"some error data").await.expect("Failed to write error file");

        let result = workspace.clone().locate_batch_files(&index).await;
        debug!("Result: {:?}", result);

        assert!(result.is_ok());
        let triple_option = result.unwrap();
        assert!(triple_option.is_some());
        let triple = triple_option.unwrap();
        assert_eq!(*triple.index(), index);
        assert!(triple.input().is_none());
        assert!(triple.output().is_none());
        assert_eq!(*triple.error(), Some(path.clone()));
        assert!(triple.associated_metadata().is_none());

        info!("Finished test: locates_single_error_file");
    }

    /// Ensures we can locate a single metadata file with no other files present.
    #[traced_test]
    async fn locates_single_metadata_file() {
        info!("Starting test: locates_single_metadata_file");
        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
        let index = BatchIndex::Usize(13);
        let filename = format!("batch_metadata_{}.jsonl", 13);

        let path = workspace.workdir().join(&filename);
        fs::write(&path, b"some metadata info").await.expect("Failed to write metadata file");

        let result = workspace.clone().locate_batch_files(&index).await;
        debug!("Result: {:?}", result);

        assert!(result.is_ok());
        let triple_option = result.unwrap();
        assert!(triple_option.is_some());
        let triple = triple_option.unwrap();
        assert_eq!(*triple.index(), index);
        assert!(triple.input().is_none());
        assert!(triple.output().is_none());
        assert!(triple.error().is_none());
        assert_eq!(*triple.associated_metadata(), Some(path.clone()));

        info!("Finished test: locates_single_metadata_file");
    }

    /// Ensures that if multiple input files exist for the same index, an error is returned.
    #[traced_test]
    async fn fails_if_multiple_input_files_found() {
        info!("Starting test: fails_if_multiple_input_files_found");
        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
        let index = BatchIndex::Usize(20);

        let filename1 = format!("batch_input_{}.jsonl", 20);
        let filename2 = format!("batch_input_{}.jsonl", 20); // same index
        let path1 = workspace.workdir().join(&filename1);
        let path2 = workspace.workdir().join(&format!("extra_{}", filename2));

        fs::write(&path1, b"first input").await.expect("Failed to write first input file");
        fs::write(&path2, b"second input").await.expect("Failed to write second input file");

        let result = workspace.clone().locate_batch_files(&index).await;
        debug!("Result: {:?}", result);

        assert!(result.is_err(), "Expected an error when multiple input files exist");
        if let Err(BatchWorkspaceError::IoError(e)) = result {
            debug!("Error kind: {:?}", e.kind());
            assert_eq!(e.kind(), std::io::ErrorKind::InvalidData, "Should produce InvalidData error");
        } else {
            panic!("Unexpected error variant");
        }
        info!("Finished test: fails_if_multiple_input_files_found");
    }

    /// Ensures that if multiple output files exist for the same index, an error is returned.
    #[traced_test]
    async fn fails_if_multiple_output_files_found() {
        info!("Starting test: fails_if_multiple_output_files_found");
        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
        let index = BatchIndex::Usize(21);

        let path1 = workspace.workdir().join(format!("batch_output_{}.jsonl", 21));
        let path2 = workspace.workdir().join(format!("some_other_batch_output_{}.jsonl", 21));

        fs::write(&path1, b"output file #1").await.expect("Failed to write output file #1");
        fs::write(&path2, b"output file #2").await.expect("Failed to write output file #2");

        let result = workspace.clone().locate_batch_files(&index).await;
        debug!("Result: {:?}", result);
        assert!(result.is_err());
        info!("Finished test: fails_if_multiple_output_files_found");
    }

    /// Ensures that if multiple error files exist for the same index, an error is returned.
    #[traced_test]
    async fn fails_if_multiple_error_files_found() {
        info!("Starting test: fails_if_multiple_error_files_found");
        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
        let index = BatchIndex::Usize(22);

        let path1 = workspace.workdir().join(format!("batch_error_{}.jsonl", 22));
        let path2 = workspace.workdir().join(format!("copy_batch_error_{}.jsonl", 22));
        fs::write(&path1, b"error file #1").await.expect("Failed to write error file #1");
        fs::write(&path2, b"error file #2").await.expect("Failed to write error file #2");

        let result = workspace.clone().locate_batch_files(&index).await;
        debug!("Result: {:?}", result);
        assert!(result.is_err(), "Expect an error due to multiple error files");
        info!("Finished test: fails_if_multiple_error_files_found");
    }

    /// Ensures that if multiple metadata files exist for the same index, an error is returned.
    #[traced_test]
    async fn fails_if_multiple_metadata_files_found() {
        info!("Starting test: fails_if_multiple_metadata_files_found");
        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
        let index = BatchIndex::Usize(23);

        let path1 = workspace.workdir().join(format!("batch_metadata_{}.jsonl", 23));
        let path2 = workspace.workdir().join(format!("meta_batch_metadata_{}.jsonl", 23));

        fs::write(&path1, b"metadata #1").await.expect("Failed to write metadata file #1");
        fs::write(&path2, b"metadata #2").await.expect("Failed to write metadata file #2");

        let result = workspace.clone().locate_batch_files(&index).await;
        debug!("Result: {:?}", result);
        assert!(result.is_err(), "Expect an error due to multiple metadata files");
        info!("Finished test: fails_if_multiple_metadata_files_found");
    }

    /// Ensures the method can handle partial sets of files (e.g., input + output, or input + error, etc.).
    #[traced_test]
    async fn finds_partial_set_of_files() {
        info!("Starting test: finds_partial_set_of_files");
        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
        let index = BatchIndex::Usize(30);

        let input_path = workspace.workdir().join(format!("batch_input_{}.jsonl", 30));
        let output_path = workspace.workdir().join(format!("batch_output_{}.jsonl", 30));
        fs::write(&input_path, b"input data").await.expect("Failed to write input file");
        fs::write(&output_path, b"output data").await.expect("Failed to write output file");

        let result = workspace.clone().locate_batch_files(&index).await;
        assert!(result.is_ok(), "Should succeed with partial set of files");
        let triple_option = result.unwrap();
        assert!(triple_option.is_some(), "Expect Some(...)");
        let triple = triple_option.unwrap();
        assert_eq!(*triple.index(), index);
        assert_eq!(*triple.input(), Some(input_path));
        assert_eq!(*triple.output(), Some(output_path));
        assert!(triple.error().is_none());
        assert!(triple.associated_metadata().is_none());

        info!("Finished test: finds_partial_set_of_files");
    }

    /// Ensures the code handles files with non-UTF-8 names gracefully by skipping them.
    #[traced_test]
    async fn gracefully_skips_non_utf8_filenames() {
        info!("Starting test: gracefully_skips_non_utf8_filenames");
        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
        let index = BatchIndex::Usize(31);

        // We'll fabricate an OsStr that isn't valid UTF-8 (on some platforms).
        // On Windows, this can be tricky, but on Unix we can do something like 0xFF byte.
        // We'll still test logic with a best attempt.
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;
            let invalid_filename = std::ffi::OsStr::from_bytes(b"batch_input_31\xFF.jsonl");
            let path = workspace.workdir().join(invalid_filename);
            let _ = std::fs::File::create(&path).expect("Failed to create non-UTF8 file");
        }

        // We'll also create a valid file
        let valid_file = workspace.workdir().join("batch_input_31.jsonl");
        fs::write(&valid_file, b"input data").await.expect("Failed to write valid input file");

        let result = workspace.clone().locate_batch_files(&index).await;
        debug!("Result: {:?}", result);

        // The presence of the valid file should yield a triple with input.
        assert!(result.is_ok(), "Should succeed, ignoring the non-UTF8 named file if any");
        let triple_option = result.unwrap();
        assert!(triple_option.is_some());
        let triple = triple_option.unwrap();
        assert_eq!(*triple.index(), index);
        assert_eq!(*triple.input(), Some(valid_file));
        info!("Finished test: gracefully_skips_non_utf8_filenames");
    }

    /// Ensures that unrecognized filenames that do match partial patterns but have invalid capturing groups are skipped.
    #[traced_test]
    async fn ignores_unrecognized_filenames() {
        info!("Starting test: ignores_unrecognized_filenames");
        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
        let index = BatchIndex::Usize(44);

        // We'll create a file that might partially match the pattern but has an unknown group
        // e.g. "batch_foo_44.jsonl" which is not input|output|error|metadata
        let path = workspace.workdir().join("batch_foo_44.jsonl");
        fs::write(&path, b"unknown type").await.expect("Failed to write unknown file");

        // Also create a valid input
        let valid_input = workspace.workdir().join("batch_input_44.jsonl");
        fs::write(&valid_input, b"some input").await.expect("Failed to write input file");

        let result = workspace.clone().locate_batch_files(&index).await;
        debug!("Result: {:?}", result);

        // We expect to find the valid input file, ignoring the "foo" file
        assert!(result.is_ok());
        let triple_option = result.unwrap();
        assert!(triple_option.is_some());
        let triple = triple_option.unwrap();
        assert_eq!(*triple.index(), index);
        assert_eq!(*triple.input(), Some(valid_input));
        assert!(triple.output().is_none());
        assert!(triple.error().is_none());
        assert!(triple.associated_metadata().is_none());
        info!("Finished test: ignores_unrecognized_filenames");
    }

    /// Ensures that the logic also works with a UUID-based index.
    #[traced_test]
    async fn locates_uuid_based_index_files() {
        info!("Starting test: locates_uuid_based_index_files");
        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let index = BatchIndex::from_uuid_str(uuid_str).expect("Failed to create batch index from uuid");

        let file_name = format!("batch_output_{}.jsonl", uuid_str);
        let path = workspace.workdir().join(&file_name);
        fs::write(&path, b"uuid output data").await.expect("Failed to write uuid-based file");

        let result = workspace.clone().locate_batch_files(&index).await;
        debug!("Result: {:?}", result);

        assert!(result.is_ok());
        let triple_option = result.unwrap();
        assert!(triple_option.is_some());
        let triple = triple_option.unwrap();
        assert_eq!(*triple.index(), index);
        assert!(triple.input().is_none());
        assert_eq!(*triple.output(), Some(path.clone()));
        assert!(triple.error().is_none());
        assert!(triple.associated_metadata().is_none());

        info!("Finished test: locates_uuid_based_index_files");
    }

    /// Ensures concurrency checks: multiple tasks calling locate_batch_files on the same workspace
    #[traced_test]
    async fn concurrent_locate_batch_files() {
        info!("Starting test: concurrent_locate_batch_files");
        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
        let index = BatchIndex::Usize(88);

        // Place one input file for index=88
        let input_name = format!("batch_input_{}.jsonl", 88);
        let input_path = workspace.workdir().join(&input_name);
        fs::write(&input_path, b"concurrent test input data").await.expect("Failed to write input file");

        // We'll spawn multiple tasks that attempt to locate batch files for the same index
        let arc_ws = workspace.clone();
        let mut tasks = Vec::new();
        for i in 0..5 {
            let ws_clone = arc_ws.clone();
            let index_clone = index.clone();
            tasks.push(tokio::spawn(async move {
                trace!("Task {} locating files for index=88", i);
                ws_clone.locate_batch_files(&index_clone).await
            }));
        }

        let results = futures::future::join_all(tasks).await;
        for (i, res) in results.into_iter().enumerate() {
            match res {
                Ok(Ok(Some(triple))) => {
                    debug!("Task {} => triple found with input: {:?}", i, triple.input());
                    assert_eq!(*triple.index(), index, "Index must match");
                }
                Ok(Ok(None)) => panic!("Task {} => unexpected None, we have an input file!", i),
                Ok(Err(e)) => panic!("Task {} => unexpected error: {:?}", i, e),
                Err(e) => panic!("Task {} => join error: {:?}", i, e),
            }
        }

        info!("Finished test: concurrent_locate_batch_files");
    }
}
