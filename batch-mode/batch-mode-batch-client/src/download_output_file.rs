// ---------------- [ File: batch-mode-batch-client/src/download_output_file.rs ]
crate::ix!();

#[async_trait]
impl<E> DownloadOutputFile<E> for BatchFileTriple
where
    E: From<BatchDownloadError>
        + From<std::io::Error>
        + From<BatchMetadataError>
        + From<OpenAIClientError>
        + Debug,
{
    async fn download_output_file(
        &mut self,
        client: &dyn LanguageModelClientInterface<E>,
    ) -> Result<(), E> {
        info!("downloading batch output file");

        // CHANGE: Instead of failing when `self.output().is_some()`,
        // we only fail if the path is actually present on disk.
        if let Some(out_path) = &self.output() {
            if out_path.exists() {
                warn!(
                    "Output file already present on disk at path={:?}. \
                     Aborting to avoid overwriting.",
                    out_path
                );
                return Err(BatchDownloadError::OutputFileAlreadyExists {
                    triple: self.clone(),
                }
                .into());
            }
        }

        // Use associated_metadata if we have it:
        let metadata_filename: PathBuf = if let Some(path) = self.associated_metadata() {
            path.clone()
        } else {
            self.effective_metadata_filename()
        };
        debug!("Using metadata file for output: {:?}", metadata_filename);

        let metadata = BatchMetadata::load_from_file(&metadata_filename).await?;
        let output_file_id = metadata.output_file_id()?; // fails if none

        let file_content = client.file_content(output_file_id).await?;

        let output_path = self.effective_output_filename();
        if let Some(parent) = output_path.parent() {
            tokio::fs::create_dir_all(parent).await.ok();
        }

        std::fs::write(&output_path, file_content)?;
        self.set_output_path(Some(output_path));

        Ok(())
    }
}

#[cfg(test)]
mod download_output_file_tests {
    use super::*;
    use futures::executor::block_on;
    use std::fs;
    use tempfile::tempdir;
    use tracing::{debug, error, info, trace, warn};

    /// Exhaustive test suite for `DownloadOutputFile` on `BatchFileTriple`.
    /// We cover scenarios of success, missing file_id, existing file, and client/IO errors.
    #[traced_test]
    async fn test_download_output_file_ok() {
        info!("Beginning test_download_output_file_ok");
        trace!("Constructing mock client...");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        debug!("Mock client: {:?}", mock_client);

        // Insert a known file in the mock so "client.file_content()" call succeeds
        let output_file_id = "some_output_file_id";
        {
            let mut files_guard = mock_client.files().write().unwrap();
            files_guard.insert(output_file_id.to_string(), Bytes::from("mock output contents"));
        }

        // Insert batch metadata
        let tmpdir = tempdir().unwrap();
        let metadata_path = tmpdir.path().join("metadata.json");
        let metadata = BatchMetadataBuilder::default()
            .batch_id("batch_for_download_output_ok".to_string())
            .input_file_id("some_input_file_id".to_string())
            .output_file_id(Some(output_file_id.to_string()))
            .error_file_id(None)
            .build()
            .unwrap();
        metadata.save_to_file(&metadata_path).await.unwrap();

        trace!("Creating BatchFileTriple with known metadata path...");
        let mut triple = BatchFileTriple::new_for_test_with_metadata_path(metadata_path.clone());
        triple.set_metadata_path(Some(metadata_path.clone()));

        // --- IMPORTANT: set the output path to the ephemeral temp dir. ---
        let out_path = tmpdir.path().join("output.json");
        triple.set_output_path(Some(out_path.clone()));

        trace!("Calling download_output_file...");
        let result = triple.download_output_file(&mock_client).await;
        debug!("Result from download_output_file: {:?}", result);

        assert!(result.is_ok(), "Should succeed for a valid output file");
        // Ensure file was written inside our tempdir
        let contents = fs::read_to_string(&out_path).unwrap();
        pretty_assert_eq!(contents, "mock output contents");

        info!("test_download_output_file_ok passed");
    }

    #[traced_test]
    async fn test_download_output_file_already_exists() {
        info!("Beginning test_download_output_file_already_exists");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        debug!("Mock client: {:?}", mock_client);

        // Insert metadata
        let tmpdir = tempdir().unwrap();
        let metadata_path = tmpdir.path().join("metadata.json");
        let metadata = BatchMetadataBuilder::default()
            .batch_id("batch_exists_output")
            .input_file_id("some_input_file_id".to_string())
            .output_file_id(Some("already_exists_output_file_id".to_string()))
            .error_file_id(None)
            .build()
            .unwrap();
        metadata.save_to_file(&metadata_path).await.unwrap();

        // Prepare the triple
        let mut triple = BatchFileTriple::new_for_test_with_metadata_path(metadata_path.clone());
        triple.set_metadata_path(Some(metadata_path.clone()));

        // Simulate that the output file is already downloaded (in the tempdir)
        let existing_output_path = tmpdir.path().join("output.json");
        fs::write(&existing_output_path, b"existing content").unwrap();
        triple.set_output_path(Some(existing_output_path.clone()));

        let result = triple.download_output_file(&mock_client).await;
        debug!("Result from download_output_file: {:?}", result);

        assert!(
            result.is_err(),
            "Should fail if output file already exists on disk"
        );
        info!("test_download_output_file_already_exists passed");
    }

    #[traced_test]
    async fn test_download_output_file_missing_output_file_id() {
        info!("Beginning test_download_output_file_missing_output_file_id");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        debug!("Mock client: {:?}", mock_client);

        // Insert metadata that does NOT have an output_file_id
        let tmpdir = tempdir().unwrap();
        let metadata_path = tmpdir.path().join("metadata.json");
        let metadata = BatchMetadataBuilder::default()
            .batch_id("batch_no_out_id")
            .input_file_id("input_file_id".to_string())
            .output_file_id(None)
            .error_file_id(None)
            .build()
            .unwrap();
        metadata.save_to_file(&metadata_path).await.unwrap();

        // Prepare the triple
        let mut triple = BatchFileTriple::new_for_test_with_metadata_path(metadata_path.clone());
        triple.set_metadata_path(Some(metadata_path.clone()));

        // --- Just to keep everything in ephemeral space, set an output path anyway ---
        let out_path = tmpdir.path().join("will_not_be_written.json");
        triple.set_output_path(Some(out_path.clone()));

        let result = triple.download_output_file(&mock_client).await;
        debug!("Result from download_output_file: {:?}", result);

        assert!(
            result.is_err(),
            "Should fail if output_file_id is not present in metadata"
        );
        info!("test_download_output_file_missing_output_file_id passed");
    }

    #[traced_test]
    async fn test_download_output_file_client_file_not_found() {
        info!("Beginning test_download_output_file_client_file_not_found");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();

        // Insert metadata referencing a nonexistent output file in the mock
        let tmpdir = tempdir().unwrap();
        let metadata_path = tmpdir.path().join("metadata.json");
        let metadata = BatchMetadataBuilder::default()
            .batch_id("batch_out_file_not_found")
            .input_file_id("some_input".to_string())
            .output_file_id(Some("out_file_that_does_not_exist".to_string()))
            .error_file_id(None)
            .build()
            .unwrap();
        metadata.save_to_file(&metadata_path).await.unwrap();

        let mut triple = BatchFileTriple::new_for_test_with_metadata_path(metadata_path.clone());
        triple.set_metadata_path(Some(metadata_path.clone()));

        // ephemeral path
        let out_path = tmpdir.path().join("output_file.json");
        triple.set_output_path(Some(out_path.clone()));

        let result = triple.download_output_file(&mock_client).await;
        debug!("Result from download_output_file: {:?}", result);

        assert!(
            result.is_err(),
            "Should fail if the mock client cannot find the output file_id"
        );
        info!("test_download_output_file_client_file_not_found passed");
    }

    #[traced_test]
    async fn test_download_output_file_io_write_error() {
        info!("Beginning test_download_output_file_io_write_error");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();

        // Put a real file in the mock, so the initial content retrieval works
        let output_file_id = "some_out_file_id_for_io_error";
        {
            let mut files_guard = mock_client.files().write().unwrap();
            files_guard.insert(output_file_id.to_string(), Bytes::from("output content"));
        }

        // We'll create one tempdir for metadata (fully writable),
        // and a separate read-only tempdir for the actual output file attempt:
        let tmpdir_meta = tempdir().unwrap();
        let tmpdir_readonly = tempdir().unwrap();

        let metadata_path = tmpdir_meta.path().join("metadata.json");
        let metadata = BatchMetadataBuilder::default()
            .batch_id("batch_io_error")
            .input_file_id("some_input".to_string())
            .output_file_id(Some(output_file_id.to_string()))
            .error_file_id(None)
            .build()
            .unwrap();
        metadata.save_to_file(&metadata_path).await.unwrap();
        debug!("Metadata saved at {:?}", metadata_path);

        // Create the triple, referencing that metadata
        let mut triple = BatchFileTriple::new_for_test_with_metadata_path(metadata_path.clone());
        triple.set_metadata_path(Some(metadata_path.clone()));

        // We forcibly set the "output file" path to the read-only dir
        let out_path = tmpdir_readonly.path().join("output.json");
        triple.set_output_path(Some(out_path.clone()));

        // Now make the directory read-only, to cause an I/O error on write:
        let mut perms = fs::metadata(tmpdir_readonly.path()).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(tmpdir_readonly.path(), perms).unwrap();

        let result = triple.download_output_file(&mock_client).await;
        debug!("Result from download_output_file: {:?}", result);

        // Revert permissions so the tempdir can be cleaned up
        let mut perms = fs::metadata(tmpdir_readonly.path()).unwrap().permissions();
        perms.set_readonly(false);
        fs::set_permissions(tmpdir_readonly.path(), perms).unwrap();

        assert!(
            result.is_err(),
            "Should fail with an I/O error when the directory is read-only"
        );
        info!("test_download_output_file_io_write_error passed");
    }
}
