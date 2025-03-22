// ---------------- [ File: src/download_error_file.rs ]
crate::ix!();

#[async_trait]
impl<E> DownloadErrorFile<E> for BatchFileTriple
where
    E: From<BatchDownloadError>
        + From<std::io::Error>
        + From<BatchMetadataError>
        + From<OpenAIClientError>
        + Debug,
{
    async fn download_error_file(
        &mut self,
        client: &dyn LanguageModelClientInterface<E>,
    ) -> Result<(), E> {
        info!("downloading batch error file");

        // CHANGE: Instead of failing when `self.error().is_some()`,
        // we only fail if the path is actually present on disk.
        if let Some(err_path) = &self.error() {
            if err_path.exists() {
                warn!(
                    "Error file already present on disk at path={:?}. \
                     Aborting to avoid overwriting.",
                    err_path
                );
                return Err(BatchDownloadError::ErrorFileAlreadyExists {
                    triple: self.clone(),
                }
                .into());
            }
        }

        let metadata_filename = match self.associated_metadata() {
            Some(file) => file.to_path_buf(),
            None => self.effective_metadata_filename().to_path_buf(),
        };
        debug!("Using metadata file for error: {:?}", metadata_filename);

        let metadata = BatchMetadata::load_from_file(&metadata_filename).await?;
        let error_file_id = metadata.error_file_id()?;

        let file_content = client.file_content(error_file_id).await?;

        let error_path = self.effective_error_filename();
        if let Some(parent) = error_path.parent() {
            tokio::fs::create_dir_all(parent).await.ok();
        }

        // Force removal if the file already exists, so no leftover content remains:
        if error_path.exists() {
            std::fs::remove_file(&error_path)?;
        }

        std::fs::write(&error_path, file_content)?;
        self.set_error_path(Some(error_path));
        Ok(())
    }
}

#[cfg(test)]
mod download_error_file_tests {
    use super::*;
    use futures::executor::block_on;
    use std::fs;
    use tempfile::tempdir;
    use tracing::{debug, error, info, trace, warn};

    /// Exhaustive test suite for `DownloadErrorFile` on `BatchFileTriple`.
    /// We'll cover scenarios of success, missing file_id, existing file, and client/IO errors.
    #[traced_test]
    async fn test_download_error_file_ok() {
        info!("Beginning test_download_error_file_ok");
        trace!("Constructing mock client...");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        debug!("Mock client: {:?}", mock_client);

        // Insert a known file in the mock so the "client.file_content()" call succeeds
        let error_file_id = "some_error_file_id";
        {
            let mut files_guard = mock_client.files().write().unwrap();
            files_guard.insert(error_file_id.to_string(), Bytes::from("mock error contents"));
        }

        // Insert batch metadata with the relevant error_file_id
        let tmpdir = tempdir().unwrap();
        let metadata_path = tmpdir.path().join("metadata.json");
        let metadata = BatchMetadataBuilder::default()
            .batch_id("some_batch_id".to_string())
            .input_file_id("some_input_file_id".to_string())
            .output_file_id(None)
            .error_file_id(Some(error_file_id.to_string()))
            .build()
            .unwrap();
        info!("Saving metadata at {:?}", metadata_path);
        metadata.save_to_file(&metadata_path).await.unwrap();

        trace!("Creating BatchFileTriple with known metadata path...");
        let mut triple = BatchFileTriple::new_for_test_with_metadata_path(metadata_path.clone());
        triple.set_metadata_path(Some(metadata_path.clone()));

        // --- Set an ephemeral error file path. ---
        let err_path = tmpdir.path().join("error.json");
        triple.set_error_path(Some(err_path.clone()));

        trace!("Calling download_error_file...");
        let result = triple.download_error_file(&mock_client).await;
        debug!("Result from download_error_file: {:?}", result);

        assert!(result.is_ok(), "Should succeed for a valid error file");
        // Ensure file was written
        let contents = fs::read_to_string(&err_path).unwrap();
        pretty_assert_eq!(contents, "mock error contents");

        info!("test_download_error_file_ok passed");
    }

    #[traced_test]
    async fn test_download_error_file_already_exists() {
        info!("Beginning test_download_error_file_already_exists");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        debug!("Mock client: {:?}", mock_client);

        // Insert metadata
        let tmpdir = tempdir().unwrap();
        let metadata_path = tmpdir.path().join("metadata.json");
        let metadata = BatchMetadataBuilder::default()
            .batch_id("batch_id_exists_err")
            .input_file_id("some_input_file_id".to_string())
            .output_file_id(None)
            .error_file_id(Some("already_exists_err_file_id".to_string()))
            .build()
            .unwrap();
        metadata.save_to_file(&metadata_path).await.unwrap();

        // Prepare the triple
        let mut triple = BatchFileTriple::new_for_test_with_metadata_path(metadata_path.clone());
        triple.set_metadata_path(Some(metadata_path.clone()));

        // Simulate that the error file is already downloaded
        let existing_err_path = tmpdir.path().join("error.json");
        fs::write(&existing_err_path, b"existing content").unwrap();
        triple.set_error_path(Some(existing_err_path.clone()));

        let result = triple.download_error_file(&mock_client).await;
        debug!("Result from download_error_file: {:?}", result);

        assert!(
            result.is_err(),
            "Should fail if error file already exists on disk"
        );
        info!("test_download_error_file_already_exists passed");
    }

    #[traced_test]
    async fn test_download_error_file_missing_error_file_id() {
        info!("Beginning test_download_error_file_missing_error_file_id");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        debug!("Mock client: {:?}", mock_client);

        // Insert metadata that does NOT have an error_file_id
        let tmpdir = tempdir().unwrap();
        let metadata_path = tmpdir.path().join("metadata.json");
        let metadata = BatchMetadataBuilder::default()
            .batch_id("batch_no_err_id")
            .input_file_id("input_file_id".to_string())
            .output_file_id(None)
            .error_file_id(None)
            .build()
            .unwrap();
        metadata.save_to_file(&metadata_path).await.unwrap();

        // Prepare the triple
        let mut triple = BatchFileTriple::new_for_test_with_metadata_path(metadata_path.clone());
        triple.set_metadata_path(Some(metadata_path.clone()));

        // ephemeral path, though it won't actually be written
        let err_path = tmpdir.path().join("placeholder_error_file.json");
        triple.set_error_path(Some(err_path.clone()));

        let result = triple.download_error_file(&mock_client).await;
        debug!("Result from download_error_file: {:?}", result);

        assert!(
            result.is_err(),
            "Should fail if error_file_id is not present in metadata"
        );
        info!("test_download_error_file_missing_error_file_id passed");
    }

    #[traced_test]
    async fn test_download_error_file_client_file_not_found() {
        info!("Beginning test_download_error_file_client_file_not_found");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();

        // Insert metadata referencing a nonexistent error file in the mock
        let tmpdir = tempdir().unwrap();
        let metadata_path = tmpdir.path().join("metadata.json");
        let metadata = BatchMetadataBuilder::default()
            .batch_id("batch_err_file_not_found")
            .input_file_id("some_input".to_string())
            .output_file_id(None)
            .error_file_id(Some("err_file_that_does_not_exist".to_string()))
            .build()
            .unwrap();
        metadata.save_to_file(&metadata_path).await.unwrap();

        let mut triple = BatchFileTriple::new_for_test_with_metadata_path(metadata_path.clone());
        triple.set_metadata_path(Some(metadata_path.clone()));

        let err_path = tmpdir.path().join("error_file.json");
        triple.set_error_path(Some(err_path.clone()));

        let result = triple.download_error_file(&mock_client).await;
        debug!("Result from download_error_file: {:?}", result);

        assert!(
            result.is_err(),
            "Should fail if the mock client cannot find the error file_id"
        );
        info!("test_download_error_file_client_file_not_found passed");
    }

    #[traced_test]
    async fn test_download_error_file_io_write_error() {
        info!("Beginning test_download_error_file_io_write_error");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();

        // Put a real file in the mock, so the content retrieval works
        let error_file_id = "some_err_file_id_for_io_error";
        {
            let mut files_guard = mock_client.files().write().unwrap();
            files_guard.insert(error_file_id.to_string(), Bytes::from("err content"));
        }

        // We'll create one tempdir for metadata (fully writable),
        // and a separate subdir for the actual error file that we make read-only.
        let tmpdir_meta = tempdir().unwrap();
        let tmpdir_readonly = tempdir().unwrap();

        // We'll keep the metadata in tmpdir_meta:
        let metadata_path = tmpdir_meta.path().join("metadata.json");
        let metadata = BatchMetadataBuilder::default()
            .batch_id("batch_io_error")
            .input_file_id("some_input".to_string())
            .output_file_id(None)
            .error_file_id(Some(error_file_id.to_string()))
            .build()
            .unwrap();
        info!("Saving metadata at {:?}", metadata_path);
        metadata.save_to_file(&metadata_path).await.unwrap();
        debug!("Metadata file created successfully.");

        // Now the triple will reference that metadata path
        let mut triple = BatchFileTriple::new_for_test_with_metadata_path(metadata_path.clone());
        triple.set_metadata_path(Some(metadata_path.clone()));

        // We forcibly set the "error file" path to the read-only dir:
        let err_path = tmpdir_readonly.path().join("error.json");
        triple.set_error_path(Some(err_path.clone()));

        // Make that directory read-only:
        let mut perms = fs::metadata(tmpdir_readonly.path()).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(tmpdir_readonly.path(), perms).unwrap();

        let result = triple.download_error_file(&mock_client).await;
        debug!("Result from download_error_file: {:?}", result);

        // Revert permissions so we can clean up tempdir_readonly:
        let mut perms = fs::metadata(tmpdir_readonly.path()).unwrap().permissions();
        perms.set_readonly(false);
        fs::set_permissions(tmpdir_readonly.path(), perms).unwrap();

        assert!(
            result.is_err(),
            "Should fail with an I/O error when the directory is read-only"
        );
        info!("test_download_error_file_io_write_error passed");
    }
}

