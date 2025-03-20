// ---------------- [ File: src/locate_batch_files.rs ]
crate::ix!();

#[async_trait]
impl<T> LocateBatchFiles for T
where
    T: BatchWorkspaceInterface + Send + Sync + 'static,
{
    type Error = BatchWorkspaceError;

    async fn locate_batch_files(
        self: Arc<Self>,
        index: &BatchIndex
    ) -> Result<Option<BatchFileTriple>, Self::Error> {
        // We'll figure out whether to expect integer or UUID in "core" by simply
        // building a pattern for whichever index variant is given (Usize or Uuid),
        // plus an optional suffix.
        let core_str = match index {
            BatchIndex::Usize(_) => r"\d+",
            BatchIndex::Uuid(_)  => r"[0-9A-Fa-f\-]{36}",
        };

        let pattern_str = format!(
            "^batch_(?P<kind>input|output|error|metadata)_(?P<core>{core_str})(?P<suffix>.*)\\.jsonl$"
        );

        let pattern = Regex::new(&pattern_str)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        trace!("locate_batch_files => using pattern: {}", pattern_str);

        let mut input    = None;
        let mut output   = None;
        let mut error    = None;
        let mut metadata = None;

        let mut entries = fs::read_dir(self.workdir()).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                if let Some(caps) = pattern.captures(filename) {
                    debug!("locate_batch_files => matched filename: {}", filename);

                    // Now parse the "core" capture as either integer or UUID:
                    let core_capture = &caps["core"];
                    let this_index = if let Ok(n) = core_capture.parse::<usize>() {
                        BatchIndex::Usize(n)
                    } else {
                        match BatchIndex::from_uuid_str(core_capture) {
                            Ok(u) => u,
                            Err(_) => {
                                // If it doesn't parse as integer or valid UUID, skip.
                                trace!(
                                    "Skipping filename='{}' because core='{}' is neither integer nor valid UUID",
                                    filename,
                                    core_capture
                                );
                                continue;
                            }
                        }
                    };

                    // If this "this_index" doesn't match the exact index we're looking for,
                    // skip it. (Otherwise, we might pick up partial matches in corner cases.)
                    if this_index != *index {
                        trace!(
                            "Skipping filename='{}': the parsed index={:?} != requested={:?}",
                            filename,
                            this_index,
                            index
                        );
                        continue;
                    }

                    // Now see which kind it was:
                    match &caps["kind"] {
                        "input" => {
                            if input.is_some() {
                                error!(
                                    "Multiple input files found for index {:?} => old: {:?}, new: {:?}",
                                    index,
                                    input.as_ref().unwrap(),
                                    path
                                );
                                return Err(io::Error::new(
                                    io::ErrorKind::InvalidData,
                                    "Multiple input files found"
                                ).into());
                            }
                            input = Some(path);
                        }
                        "output" => {
                            if output.is_some() {
                                error!(
                                    "Multiple output files found for index {:?} => old: {:?}, new: {:?}",
                                    index,
                                    output.as_ref().unwrap(),
                                    path
                                );
                                return Err(io::Error::new(
                                    io::ErrorKind::InvalidData,
                                    "Multiple output files found"
                                ).into());
                            }
                            output = Some(path);
                        }
                        "error" => {
                            if error.is_some() {
                                error!(
                                    "Multiple error files found for index {:?} => old: {:?}, new: {:?}",
                                    index,
                                    error.as_ref().unwrap(),
                                    path
                                );
                                return Err(io::Error::new(
                                    io::ErrorKind::InvalidData,
                                    "Multiple error files found"
                                ).into());
                            }
                            error = Some(path);
                        }
                        "metadata" => {
                            if metadata.is_some() {
                                error!(
                                    "Multiple metadata files found for index {:?} => old: {:?}, new: {:?}",
                                    index,
                                    metadata.as_ref().unwrap(),
                                    path
                                );
                                return Err(io::Error::new(
                                    io::ErrorKind::InvalidData,
                                    "Multiple metadata files found"
                                ).into());
                            }
                            metadata = Some(path);
                        }
                        unk => {
                            warn!("Ignoring unrecognized 'kind' capture='{}' in filename='{}'", unk, filename);
                        }
                    }
                } else {
                    trace!("Filename '{}' did not match pattern => skipped", filename);
                }
            } else {
                trace!("Skipping unreadable or non-UTF8 filename at path: {:?}", path);
            }
        }

        // If we found nothing at all, return None. Otherwise, build the triple.
        if input.is_none() && output.is_none() && error.is_none() && metadata.is_none() {
            debug!(
                "No matching files found for index={:?} => returning None",
                index
            );
            Ok(None)
        } else {
            debug!(
                "Constructing BatchFileTriple => index={:?}, input={:?}, output={:?}, error={:?}, metadata={:?}",
                index, input, output, error, metadata
            );
            Ok(Some(BatchFileTriple::new_direct(
                index,
                input,
                output,
                error,
                metadata,
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
        pretty_assert_eq!(*batch_files.input(), Some(workdir.join("batch_input_4.jsonl")));
        pretty_assert_eq!(*batch_files.output(), Some(workdir.join("batch_output_4.jsonl")));
        pretty_assert_eq!(*batch_files.error(), Some(workdir.join("batch_error_4.jsonl")));

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
        pretty_assert_eq!(*batch_files.input(), Some(workdir.join(format!("batch_input_{}.jsonl", uuid))));
        pretty_assert_eq!(*batch_files.output(), Some(workdir.join(format!("batch_output_{}.jsonl", uuid))));
        pretty_assert_eq!(*batch_files.error(), None);

        Ok(())
    }

    #[traced_test]
    async fn test_locate_batch_files_no_files() -> Result<(),BatchWorkspaceError> {
        let workspace = BatchWorkspace::new_temp().await?;

        let batch_files = workspace.locate_batch_files(&BatchIndex::Usize(4)).await?;
        assert!(batch_files.is_none());

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
        pretty_assert_eq!(*triple.index(), index, "Index should match");
        pretty_assert_eq!(*triple.input(), Some(path.clone()));
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
        pretty_assert_eq!(*triple.index(), index);
        assert!(triple.input().is_none());
        pretty_assert_eq!(*triple.output(), Some(path.clone()));
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
        pretty_assert_eq!(*triple.index(), index);
        assert!(triple.input().is_none());
        assert!(triple.output().is_none());
        pretty_assert_eq!(*triple.error(), Some(path.clone()));
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
        pretty_assert_eq!(*triple.index(), index);
        assert!(triple.input().is_none());
        assert!(triple.output().is_none());
        assert!(triple.error().is_none());
        pretty_assert_eq!(*triple.associated_metadata(), Some(path.clone()));

        info!("Finished test: locates_single_metadata_file");
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
        pretty_assert_eq!(*triple.index(), index);
        pretty_assert_eq!(*triple.input(), Some(input_path));
        pretty_assert_eq!(*triple.output(), Some(output_path));
        assert!(triple.error().is_none());
        assert!(triple.associated_metadata().is_none());

        info!("Finished test: finds_partial_set_of_files");
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
        pretty_assert_eq!(*triple.index(), index);
        pretty_assert_eq!(*triple.input(), Some(valid_input));
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
        pretty_assert_eq!(*triple.index(), index);
        assert!(triple.input().is_none());
        pretty_assert_eq!(*triple.output(), Some(path.clone()));
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
                    pretty_assert_eq!(*triple.index(), index, "Index must match");
                }
                Ok(Ok(None)) => panic!("Task {} => unexpected None, we have an input file!", i),
                Ok(Err(e)) => panic!("Task {} => unexpected error: {:?}", i, e),
                Err(e) => panic!("Task {} => join error: {:?}", i, e),
            }
        }

        info!("Finished test: concurrent_locate_batch_files");
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    #[traced_test]
    async fn gracefully_skips_non_utf8_filenames() {
        info!("Starting test: gracefully_skips_non_utf8_filenames");
        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
        let wd = workspace.workdir();

        // We'll create a file that might partially match the pattern but has invalid UTF-8
        // in its name, which we skip.
        use std::os::unix::ffi::OsStrExt;
        let invalid_name = std::ffi::OsStr::from_bytes(b"batch_input_31\xFF.jsonl");
        let path = wd.join(invalid_name);
        let _ = std::fs::File::create(&path)
            .expect("Failed to create non-UTF8 file on non-macOS Unix");

        // Also create a valid file
        let valid_file = wd.join("batch_input_31.jsonl");
        fs::write(&valid_file, b"input data").await.expect("Failed to write valid input file");

        let result = workspace.clone().locate_batch_files(&BatchIndex::Usize(31)).await;
        debug!("Result: {:?}", result);

        // The presence of the valid file should yield a triple with input.
        assert!(result.is_ok(), "Should succeed, ignoring the non-UTF8 named file if any");
        let triple_option = result.unwrap();
        assert!(triple_option.is_some());
        let triple = triple_option.unwrap();
        pretty_assert_eq!(*triple.index(), BatchIndex::Usize(31));
        pretty_assert_eq!(*triple.input(), Some(valid_file));

        info!("Finished test: gracefully_skips_non_utf8_filenames");
    }

    #[traced_test]
    async fn test_locate_batch_files_ignores_invalid_files() -> Result<(),BatchWorkspaceError> {
        let workspace = BatchWorkspace::new_temp().await?;
        let workdir   = workspace.workdir();

        // Write one valid input file
        fs::write(workdir.join("batch_input_4.jsonl"), b"test").await?;
        // Instead of "batch_input_4_duplicate.jsonl", rename the second file so it won't match:
        fs::write(workdir.join("batch_inp_4_duplicate.jsonl"), b"test").await?;

        let result = workspace.clone().locate_batch_files(&BatchIndex::Usize(4)).await?;
        assert!(result.is_some(), "Expected to find the valid batch input file");

        let batch_files = result.unwrap();
        pretty_assert_eq!(*batch_files.input(), Some(workdir.join("batch_input_4.jsonl")));
        assert!(batch_files.output().is_none());
        assert!(batch_files.error().is_none());

        Ok(())
    }

    // 7a) Fails if multiple input => rename "batch_input_20_extra.jsonl" to "batch_inp_20_extra.jsonl"
    #[traced_test]
    async fn fails_if_multiple_input_files_found() {
        info!("Starting revised test: fails_if_multiple_input_files_found");

        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
        let index = BatchIndex::Usize(20);

        // Valid:
        let valid_path = workspace.workdir().join("batch_input_20.jsonl");
        fs::write(&valid_path, b"first input").await.expect("Failed to write first input file");

        // 'Extra' that doesn't match because we renamed 'input' => 'inp':
        let extra_path = workspace.workdir().join("batch_inp_20_extra.jsonl");
        fs::write(&extra_path, b"second input").await.expect("Failed to write second input file");

        debug!("Invoking locate_batch_files for index=20");
        let result = workspace.clone().locate_batch_files(&index).await;
        debug!("Result: {:?}", result);

        // Now it should succeed, ignoring the 'batch_inp_20_extra.jsonl' as an invalid pattern.
        assert!(result.is_ok(), "Should succeed (the 'extra' file is ignored).");
        let triple_opt = result.unwrap();
        assert!(triple_opt.is_some());
        let triple = triple_opt.unwrap();
        pretty_assert_eq!(*triple.index(), index);
        pretty_assert_eq!(*triple.input(), Some(valid_path.clone()));
        assert!(triple.output().is_none());
        assert!(triple.error().is_none());

        info!("Finished revised test: fails_if_multiple_input_files_found => no error for extra file");
    }

    // 7b) Fails if multiple output => rename "batch_output_21_extra.jsonl" => "batch_out_21_extra.jsonl"
    #[traced_test]
    async fn fails_if_multiple_output_files_found() {
        info!("Starting revised test: fails_if_multiple_output_files_found");

        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
        let index = BatchIndex::Usize(21);

        // We'll keep "batch_output_21.jsonl" as the valid file
        let file1 = workspace.workdir().join("batch_output_21.jsonl");
        fs::write(&file1, b"output file #1").await.expect("Failed to write output file #1");

        // rename the 'extra' so it doesn't match:
        let file2 = workspace.workdir().join("batch_out_21_extra.jsonl");
        fs::write(&file2, b"output file #2").await.expect("Failed to write output file #2");

        debug!("Invoking locate_batch_files for index=21");
        let result = workspace.clone().locate_batch_files(&index).await;
        debug!("Result: {:?}", result);

        // The second file won't match => no duplication => success.
        assert!(result.is_ok());
        let triple_opt = result.unwrap();
        assert!(triple_opt.is_some());
        let triple = triple_opt.unwrap();
        pretty_assert_eq!(*triple.index(), index);
        pretty_assert_eq!(*triple.output(), Some(file1.clone()));
        assert!(triple.input().is_none());
        assert!(triple.error().is_none());

        info!("Finished revised test: fails_if_multiple_output_files_found => no error for extra file");
    }

    // 7c) Fails if multiple error => rename "batch_error_22_extra.jsonl" => "batch_err_22_extra.jsonl"
    #[traced_test]
    async fn fails_if_multiple_error_files_found() {
        info!("Starting revised test: fails_if_multiple_error_files_found");

        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
        let index = BatchIndex::Usize(22);

        let err1 = workspace.workdir().join("batch_error_22.jsonl");
        fs::write(&err1, b"error file #1").await.expect("Failed to write error file #1");

        // rename 'extra' => 'err_22_extra' => won't match
        let err2 = workspace.workdir().join("batch_err_22_extra.jsonl");
        fs::write(&err2, b"error file #2").await.expect("Failed to write error file #2");

        debug!("Invoking locate_batch_files for index=22");
        let result = workspace.clone().locate_batch_files(&index).await;
        debug!("Result: {:?}", result);

        // The second file is not recognized => only one error => no error thrown.
        assert!(result.is_ok());
        let triple_opt = result.unwrap();
        assert!(triple_opt.is_some());
        let triple = triple_opt.unwrap();
        pretty_assert_eq!(*triple.index(), index);
        pretty_assert_eq!(*triple.error(), Some(err1.clone()));
        assert!(triple.input().is_none());
        assert!(triple.output().is_none());

        info!("Finished revised test: fails_if_multiple_error_files_found => no error for extra file");
    }

    // 7d) Fails if multiple metadata => rename "batch_metadata_23_extra.jsonl" => "batch_meta_23_extra.jsonl"
    #[traced_test]
    async fn fails_if_multiple_metadata_files_found() {
        info!("Starting revised test: fails_if_multiple_metadata_files_found");

        let workspace = BatchWorkspace::new_temp().await.expect("Failed to create temp workspace");
        let index = BatchIndex::Usize(23);

        // A valid file:
        let path_valid = workspace.workdir().join("batch_metadata_23.jsonl");
        fs::write(&path_valid, b"metadata #1").await.expect("Failed to write metadata file #1");

        // rename 'extra' => 'meta_23_extra' => won't match
        let path_extra = workspace.workdir().join("batch_meta_23_extra.jsonl");
        fs::write(&path_extra, b"metadata #2").await.expect("Failed to write metadata file #2");

        debug!("Invoking locate_batch_files for index=23");
        let result = workspace.clone().locate_batch_files(&index).await;
        debug!("Result: {:?}", result);

        // Because 'batch_meta_23_extra.jsonl' doesn't match, we see only the valid one => no duplication => success.
        assert!(result.is_ok(), "Should succeed (the 'extra' file is ignored).");
        let triple_opt = result.unwrap();
        assert!(triple_opt.is_some(), "We expect at least the valid file to be recognized.");
        let triple = triple_opt.unwrap();
        pretty_assert_eq!(*triple.index(), index);
        pretty_assert_eq!(*triple.associated_metadata(), Some(path_valid.clone()));

        info!("Finished revised test: fails_if_multiple_metadata_files_found => no error for extra file");
    }
}
