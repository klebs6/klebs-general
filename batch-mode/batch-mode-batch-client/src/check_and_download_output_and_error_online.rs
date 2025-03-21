// ---------------- [ File: src/check_and_download_output_and_error_online.rs ]
crate::ix!();

#[async_trait]
impl<E> CheckForAndDownloadOutputAndErrorOnline<E> for BatchFileTriple
where
    E: From<BatchDownloadError>
        + From<OpenAIClientError>
        + From<BatchMetadataError>
        + From<std::io::Error> 
        + Debug
        + Display,
{
    async fn check_for_and_download_output_and_error_online(
        &mut self,
        client: &dyn LanguageModelClientInterface<E>,
    ) -> Result<(), E> {
        trace!("Entered check_for_and_download_output_and_error_online.");
        info!("Checking for and downloading output/error files if available.");

        // If we are incomplete, or have a failure, check_batch_status_online returns an error.
        let status = match self.check_batch_status_online(client).await {
            Ok(s) => {
                debug!("Successfully retrieved batch status online.");
                s
            }
            Err(e) => {
                error!("Failed to retrieve batch status online. {e}");
                return Err(e);
            }
        };

        info!("Batch online status: {:?}", status);

        if status.output_file_available() {
            debug!("Output file is available; attempting to download...");
            if let Err(e) = self.download_output_file(client).await {
                error!("Failed to download output file. {e}");
                return Err(e);
            }
            debug!("Successfully downloaded output file.");
        } else {
            trace!("No output file available for download.");
        }

        if status.error_file_available() {
            debug!("Error file is available; attempting to download...");
            if let Err(e) = self.download_error_file(client).await {
                error!("Failed to download error file. {e}");
                return Err(e);
            }
            debug!("Successfully downloaded error file.");
        } else {
            trace!("No error file available for download.");
        }

        info!("Completed check_for_and_download_output_and_error_online successfully.");
        Ok(())
    }
}

#[cfg(test)]
mod check_for_and_download_output_and_error_online_tests {
    use super::*;
    use futures::executor::block_on;
    use std::fs;
    use tempfile::tempdir;
    use tracing::{debug, error, info, trace, warn};

    #[traced_test]
    async fn test_completed_with_output_only() {
        info!("Beginning test_completed_with_output_only");
        trace!("Constructing mock client for completed batch with output only...");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        debug!("Mock client constructed: {:?}", mock_client);

        let batch_id = "batch_completed_output_only";
        {
            let mut guard = mock_client.batches().write().unwrap();
            guard.insert(
                batch_id.to_string(),
                Batch {
                    id: batch_id.to_string(),
                    object: "batch".to_string(),
                    endpoint: "/v1/chat/completions".to_string(),
                    errors: None,
                    input_file_id: "some_input_file".to_string(),
                    completion_window: "24h".to_string(),
                    status: BatchStatus::Completed,
                    output_file_id: Some("mock_output_file_id".to_string()),
                    error_file_id: None,
                    created_at: 0,
                    in_progress_at: None,
                    expires_at: None,
                    finalizing_at: None,
                    completed_at: None,
                    failed_at: None,
                    expired_at: None,
                    cancelling_at: None,
                    cancelled_at: None,
                    request_counts: None,
                    metadata: None,
                },
            );
        }
        debug!("Mock batch inserted with status: Completed, output only");

        trace!("Mocking file contents for output file: mock_output_file_id");
        {
            let mut files_guard = mock_client.files().write().unwrap();
            // The actual downloadable content:
            files_guard.insert("mock_output_file_id".to_string(), Bytes::from("mock output data"));
        }

        trace!("Creating temp dir and saving metadata...");
        let tmp_dir = tempdir().unwrap();
        let metadata_path = tmp_dir.path().join("metadata.json");
        let metadata = BatchMetadataBuilder::default()
            .batch_id(batch_id.to_string())
            .input_file_id("some_input_file".to_string())
            .output_file_id(Some("mock_output_file_id".into()))
            .error_file_id(None)
            .build()
            .unwrap();
        info!("Saving metadata at {:?}", metadata_path);
        metadata.save_to_file(&metadata_path).await.unwrap();
        debug!("Metadata saved to {:?}", metadata_path);

        trace!("Constructing BatchFileTriple and ensuring we use the correct metadata path...");
        let mut triple = BatchFileTriple::new_for_test_with_metadata_path(metadata_path.clone());
        // IMPORTANT: We do NOT pre‐write any existing file content here, so we can truly test the fresh download.
        triple.set_metadata_path(Some(metadata_path.clone()));

        trace!("Calling check_for_and_download_output_and_error_online...");
        let result = triple
            .check_for_and_download_output_and_error_online(&mock_client)
            .await;
        debug!("Result from check call: {:?}", result);

        assert!(
            result.is_ok(),
            "Should succeed for completed batch with output only"
        );

        // Confirm the downloaded file has the new mock content
        let output_filename = triple.effective_output_filename();
        let contents = std::fs::read_to_string(&output_filename).unwrap();
        pretty_assert_eq!(contents, "mock output data");

        info!("test_completed_with_output_only passed");
    }

    #[traced_test]
    async fn test_incomplete_batch_returns_error() {
        info!("Beginning test_incomplete_batch_returns_error");
        trace!("Constructing mock client for incomplete batch scenario...");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        debug!("Mock client constructed: {:?}", mock_client);

        let batch_id = "batch_in_progress";
        trace!("Inserting mock batch with ID: {}", batch_id);
        {
            let mut guard = mock_client.batches().write().unwrap();
            guard.insert(
                batch_id.to_string(),
                Batch {
                    id: batch_id.to_string(),
                    object: "batch".to_string(),
                    endpoint: "/v1/chat/completions".to_string(),
                    errors: None,
                    input_file_id: "some_input_file".to_string(),
                    output_file_id: Some("some_output_file".to_string()),
                    completion_window: "24h".to_string(),
                    status: BatchStatus::InProgress,
                    error_file_id: None,
                    created_at: 0,
                    in_progress_at: None,
                    expires_at: None,
                    finalizing_at: None,
                    completed_at: None,
                    failed_at: None,
                    expired_at: None,
                    cancelling_at: None,
                    cancelled_at: None,
                    request_counts: None,
                    metadata: None,
                },
            );
        }
        debug!("Mock batch inserted with status: InProgress");

        trace!("Creating temp dir and saving metadata...");
        let tmp_dir = tempdir().unwrap();
        let metadata_path = tmp_dir.path().join("metadata.json");
        let metadata = BatchMetadataBuilder::default()
            .batch_id(batch_id.to_string())
            .input_file_id("some_input_file".to_string())
            .output_file_id("some_output_file".to_string())
            .error_file_id(None)
            .build()
            .unwrap();
        metadata.save_to_file(&metadata_path).await.unwrap();
        debug!("Metadata saved to {:?}", metadata_path);

        trace!("Constructing BatchFileTriple and calling check_for_and_download_output_and_error_online...");
        let mut triple = BatchFileTriple::new_for_test_with_metadata_path(metadata_path.clone());
        let result = triple
            .check_for_and_download_output_and_error_online(&mock_client)
            .await;
        debug!("Result from check call: {:?}", result);

        assert!(result.is_err(), "Should fail if batch is incomplete");
        info!("test_incomplete_batch_returns_error passed");
    }

    #[traced_test]
    async fn test_failed_batch_returns_error() {
        info!("Beginning test_failed_batch_returns_error");
        trace!("Constructing mock client for failed batch scenario...");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        debug!("Mock client constructed: {:?}", mock_client);

        let batch_id = "batch_failed";
        trace!("Inserting mock batch with ID: {}", batch_id);
        {
            let mut guard = mock_client.batches().write().unwrap();
            guard.insert(
                batch_id.to_string(),
                Batch {
                    id: batch_id.to_string(),
                    object: "batch".to_string(),
                    endpoint: "/v1/chat/completions".to_string(),
                    errors: None,
                    input_file_id: "some_input_file".to_string(),
                    completion_window: "24h".to_string(),
                    status: BatchStatus::Failed,
                    output_file_id: Some("some_output_file".to_string()),
                    error_file_id: None,
                    created_at: 0,
                    in_progress_at: None,
                    expires_at: None,
                    finalizing_at: None,
                    completed_at: None,
                    failed_at: None,
                    expired_at: None,
                    cancelling_at: None,
                    cancelled_at: None,
                    request_counts: None,
                    metadata: None,
                },
            );
        }
        debug!("Mock batch inserted with status: Failed");

        trace!("Creating temp dir and saving metadata...");
        let tmp_dir = tempdir().unwrap();
        let metadata_path = tmp_dir.path().join("metadata.json");
        let metadata = BatchMetadataBuilder::default()
            .batch_id(batch_id.to_string())
            .input_file_id("some_input_file".to_string())
            .output_file_id("some_output_file".to_string())
            .error_file_id(None)
            .build()
            .unwrap();
        metadata.save_to_file(&metadata_path).await.unwrap();
        debug!("Metadata saved to {:?}", metadata_path);

        trace!("Constructing BatchFileTriple and calling check_for_and_download_output_and_error_online...");
        let mut triple = BatchFileTriple::new_for_test_with_metadata_path(metadata_path.clone());
        let result = triple
            .check_for_and_download_output_and_error_online(&mock_client)
            .await;
        debug!("Result from check call: {:?}", result);

        assert!(result.is_err(), "Should fail if batch status is Failed");
        info!("test_failed_batch_returns_error passed");
    }

    #[traced_test]
    async fn test_output_file_already_exists() {
        info!("Beginning test_output_file_already_exists");
        trace!("Constructing mock client for completed batch where output already exists...");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        debug!("Mock client constructed: {:?}", mock_client);

        let batch_id = "batch_completed_out_exists";
        let output_file_id = "mock_out_id_exists";
        trace!("Inserting mock batch with ID: {}", batch_id);
        {
            let mut guard = mock_client.batches().write().unwrap();
            guard.insert(
                batch_id.to_string(),
                Batch {
                    id: batch_id.to_string(),
                    object: "batch".to_string(),
                    endpoint: "/v1/chat/completions".to_string(),
                    errors: None,
                    input_file_id: "some_input_file".to_string(),
                    completion_window: "24h".to_string(),
                    status: BatchStatus::Completed,
                    output_file_id: Some(output_file_id.to_string()),
                    error_file_id: None,
                    created_at: 0,
                    in_progress_at: None,
                    expires_at: None,
                    finalizing_at: None,
                    completed_at: None,
                    failed_at: None,
                    expired_at: None,
                    cancelling_at: None,
                    cancelled_at: None,
                    request_counts: None,
                    metadata: None,
                },
            );
        }
        debug!("Mock batch inserted with status: Completed, output file ID: {}", output_file_id);

        trace!("Mocking file contents for output file: {}", output_file_id);
        {
            let mut files_guard = mock_client.files().write().unwrap();
            files_guard.insert(output_file_id.to_string(), Bytes::from("mock out data"));
        }

        trace!("Creating temp dir and saving metadata...");
        let tmp_dir = tempdir().unwrap();
        let metadata_path = tmp_dir.path().join("metadata.json");
        let metadata = BatchMetadataBuilder::default()
            .batch_id(batch_id.to_string())
            .input_file_id("some_input_file".to_string())
            .output_file_id(Some(output_file_id.to_string()))
            .error_file_id(None)
            .build()
            .unwrap();
        metadata.save_to_file(&metadata_path).await.unwrap();
        debug!("Metadata saved to {:?}", metadata_path);

        trace!("Constructing BatchFileTriple; simulating pre-existing output file...");
        let mut triple = BatchFileTriple::new_for_test_with_metadata_path(metadata_path.clone());
        let out_path = triple.effective_output_filename();
        fs::write(&out_path, b"Existing content").unwrap();
        triple.set_output_path(Some(out_path));
        debug!("Output file forcibly pre-created at {:?}", triple.output());

        trace!("Calling check_for_and_download_output_and_error_online...");
        let result = triple
            .check_for_and_download_output_and_error_online(&mock_client)
            .await;
        debug!("Result from check call: {:?}", result);

        assert!(result.is_err(), "Should fail if output file already exists");
        info!("test_output_file_already_exists passed");
    }

    #[traced_test]
    async fn test_error_file_already_exists() {
        info!("Beginning test_error_file_already_exists");
        trace!("Constructing mock client for completed batch where error file already exists...");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        debug!("Mock client constructed: {:?}", mock_client);

        let batch_id = "batch_completed_err_exists";
        let error_file_id = "mock_err_id_exists";
        trace!("Inserting mock batch with ID: {}", batch_id);
        {
            let mut guard = mock_client.batches().write().unwrap();
            guard.insert(
                batch_id.to_string(),
                Batch {
                    id: batch_id.to_string(),
                    object: "batch".to_string(),
                    endpoint: "/v1/chat/completions".to_string(),
                    errors: None,
                    input_file_id: "some_input_file".to_string(),
                    completion_window: "24h".to_string(),
                    status: BatchStatus::Completed,
                    output_file_id: Some("some_output_file".to_string()),
                    error_file_id: Some(error_file_id.to_string()),
                    created_at: 0,
                    in_progress_at: None,
                    expires_at: None,
                    finalizing_at: None,
                    completed_at: None,
                    failed_at: None,
                    expired_at: None,
                    cancelling_at: None,
                    cancelled_at: None,
                    request_counts: None,
                    metadata: None,
                },
            );
        }
        debug!(
            "Mock batch inserted with status: Completed, error file ID: {}",
            error_file_id
        );

        trace!("Mocking file contents for error file: {}", error_file_id);
        {
            let mut files_guard = mock_client.files().write().unwrap();
            files_guard.insert(error_file_id.to_string(), Bytes::from("mock err data"));
        }

        trace!("Creating temp dir and saving metadata...");
        let tmp_dir = tempdir().unwrap();
        let metadata_path = tmp_dir.path().join("metadata.json");
        let metadata = BatchMetadataBuilder::default()
            .batch_id(batch_id.to_string())
            .input_file_id("some_input_file".to_string())
            .output_file_id("some_output_file".to_string())
            .error_file_id(Some(error_file_id.to_string()))
            .build()
            .unwrap();
        metadata.save_to_file(&metadata_path).await.unwrap();
        debug!("Metadata saved to {:?}", metadata_path);

        trace!("Constructing BatchFileTriple; simulating pre-existing error file...");
        let mut triple = BatchFileTriple::new_for_test_with_metadata_path(metadata_path.clone());
        let err_path = triple.effective_error_filename();
        fs::write(&err_path, b"Existing error content").unwrap();
        triple.set_error_path(Some(err_path));
        debug!("Error file forcibly pre-created at {:?}", triple.error());

        trace!("Calling check_for_and_download_output_and_error_online...");
        let result = triple
            .check_for_and_download_output_and_error_online(&mock_client)
            .await;
        debug!("Result from check call: {:?}", result);

        assert!(result.is_err(), "Should fail if error file already exists");
        info!("test_error_file_already_exists passed");
    }

    #[traced_test]
    async fn test_completed_no_output_no_error() {
        info!("Beginning test_completed_no_output_no_error");
        trace!("Constructing mock client for completed batch with no files...");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        debug!("Mock client constructed: {:?}", mock_client);

        let batch_id = "batch_completed_no_files";
        trace!("Inserting mock batch with ID: {}", batch_id);
        {
            let mut guard = mock_client.batches().write().unwrap();
            guard.insert(
                batch_id.to_string(),
                Batch {
                    id: batch_id.to_string(),
                    object: "batch".to_string(),
                    endpoint: "/v1/chat/completions".to_string(),
                    errors: None,
                    input_file_id: "some_input_file".to_string(),
                    completion_window: "24h".to_string(),
                    status: BatchStatus::Completed,
                    output_file_id: None,
                    error_file_id: None,
                    created_at: 0,
                    in_progress_at: None,
                    expires_at: None,
                    finalizing_at: None,
                    completed_at: None,
                    failed_at: None,
                    expired_at: None,
                    cancelling_at: None,
                    cancelled_at: None,
                    request_counts: None,
                    metadata: None,
                },
            );
        }
        debug!("Mock batch inserted with status: Completed, no output/error files");

        trace!("Creating temp dir and saving metadata...");
        let tmp_dir = tempdir().unwrap();
        let metadata_path = tmp_dir.path().join("metadata.json");
        let metadata = BatchMetadataBuilder::default()
            .batch_id(batch_id.to_string())
            .input_file_id("some_input_file".to_string())
            .output_file_id(None)
            .error_file_id(None)
            .build()
            .unwrap();
        metadata.save_to_file(&metadata_path).await.unwrap();
        debug!("Metadata saved to {:?}", metadata_path);

        trace!("Constructing BatchFileTriple and ensuring we use the correct metadata path...");
        let mut triple = BatchFileTriple::new_for_test_with_metadata_path(metadata_path.clone());
        // Ensure the triple uses our test metadata path (prevents fallback to 'mock_metadata_9999.json'):
        triple.set_metadata_path(Some(metadata_path.clone()));

        trace!("Calling check_for_and_download_output_and_error_online...");
        let result = triple
            .check_for_and_download_output_and_error_online(&mock_client)
            .await;
        debug!("Result from check call: {:?}", result);

        assert!(
            result.is_ok(),
            "Should succeed if batch is Completed with no output or error files"
        );
        info!("test_completed_no_output_no_error passed");
    }

    #[traced_test]
    async fn test_completed_with_output_only() {
        info!("Beginning test_completed_with_output_only");
        trace!("Constructing mock client for completed batch with output only...");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        debug!("Mock client constructed: {:?}", mock_client);

        let batch_id = "batch_completed_output_only";
        let output_file_id = "mock_output_file_id";
        trace!("Inserting mock batch with ID: {}", batch_id);
        {
            let mut guard = mock_client.batches().write().unwrap();
            guard.insert(
                batch_id.to_string(),
                Batch {
                    id: batch_id.to_string(),
                    object: "batch".to_string(),
                    endpoint: "/v1/chat/completions".to_string(),
                    errors: None,
                    input_file_id: "some_input_file".to_string(),
                    completion_window: "24h".to_string(),
                    status: BatchStatus::Completed,
                    output_file_id: Some(output_file_id.to_string()),
                    error_file_id: None,
                    created_at: 0,
                    in_progress_at: None,
                    expires_at: None,
                    finalizing_at: None,
                    completed_at: None,
                    failed_at: None,
                    expired_at: None,
                    cancelling_at: None,
                    cancelled_at: None,
                    request_counts: None,
                    metadata: None,
                },
            );
        }
        debug!("Mock batch inserted with status: Completed, output only");

        trace!("Mocking file contents for output file: {}", output_file_id);
        {
            let mut files_guard = mock_client.files().write().unwrap();
            // IMPORTANT: Must match what the test ultimately asserts:
            files_guard.insert(output_file_id.to_string(), Bytes::from("mock output data"));
        }

        trace!("Creating temp dir and saving metadata...");
        let tmp_dir = tempdir().unwrap();
        let metadata_path = tmp_dir.path().join("metadata.json");
        let metadata = BatchMetadataBuilder::default()
            .batch_id(batch_id.to_string())
            .input_file_id("some_input_file".to_string())
            .output_file_id(Some(output_file_id.to_string()))
            .error_file_id(None)
            .build()
            .unwrap();
        metadata.save_to_file(&metadata_path).await.unwrap();
        debug!("Metadata saved to {:?}", metadata_path);

        trace!("Constructing BatchFileTriple and ensuring we use the correct metadata path...");
        let mut triple = BatchFileTriple::new_for_test_with_metadata_path(metadata_path.clone());
        triple.set_metadata_path(Some(metadata_path.clone()));

        trace!("Calling check_for_and_download_output_and_error_online...");
        let result = triple
            .check_for_and_download_output_and_error_online(&mock_client)
            .await;
        debug!("Result from check call: {:?}", result);

        assert!(
            result.is_ok(),
            "Should succeed for completed batch with output only"
        );
        let output_filename = triple.effective_output_filename();
        let contents = std::fs::read_to_string(&output_filename).unwrap();
        pretty_assert_eq!(contents, "mock output data");

        info!("test_completed_with_output_only passed");
    }

    #[traced_test]
    async fn test_completed_with_error_only() {
        info!("Beginning test_completed_with_error_only");
        trace!("Constructing mock client for completed batch with error only...");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        debug!("Mock client constructed: {:?}", mock_client);

        let batch_id = "batch_completed_error_only";
        {
            // Insert the batch as Completed w/ error only
            let mut guard = mock_client.batches().write().unwrap();
            guard.insert(
                batch_id.to_string(),
                Batch {
                    id: batch_id.to_string(),
                    object: "batch".to_string(),
                    endpoint: "/v1/chat/completions".to_string(),
                    errors: None,
                    input_file_id: "some_input_file".to_string(),
                    completion_window: "24h".to_string(),
                    status: BatchStatus::Completed,
                    output_file_id: None,
                    error_file_id: Some("mock_error_file_id".to_string()),
                    created_at: 0,
                    in_progress_at: None,
                    expires_at: None,
                    finalizing_at: None,
                    completed_at: None,
                    failed_at: None,
                    expired_at: None,
                    cancelling_at: None,
                    cancelled_at: None,
                    request_counts: None,
                    metadata: None,
                },
            );
        }
        debug!("Mock batch inserted with status: Completed, error only");

        // Provide an actual file in the mock
        {
            let mut files_guard = mock_client.files().write().unwrap();
            files_guard.insert("mock_error_file_id".into(), Bytes::from("mock error data"));
        }

        // Create the local metadata
        let tmpdir = tempdir().unwrap();
        let metadata_path = tmpdir.path().join("metadata.json");
        let metadata = BatchMetadataBuilder::default()
            .batch_id(batch_id.to_string())
            .input_file_id("some_input_file".to_string())
            .output_file_id(None)
            .error_file_id(Some("mock_error_file_id".to_string()))
            .build()
            .unwrap();
        metadata.save_to_file(&metadata_path).await.unwrap();

        let mut triple = BatchFileTriple::new_for_test_with_metadata_path_unique(metadata_path.clone());
        // ^ changed here from new_for_test_with_metadata_path to ..._unique
        triple.set_metadata_path(Some(metadata_path.clone()));

        let result = triple
            .check_for_and_download_output_and_error_online(&mock_client)
            .await;
        debug!("Result => {:?}", result);
        assert!(result.is_ok(), "Should succeed for completed batch w/ error only");

        // Confirm the file was actually written and contains the correct data
        let error_filename = triple.effective_error_filename();
        let contents = std::fs::read_to_string(&error_filename).unwrap();
        pretty_assert_eq!(contents, "mock error data");  // was "existing content" before fix
    }

    #[traced_test]
    async fn test_completed_with_both_output_and_error() {
        info!("Beginning test_completed_with_both_output_and_error");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();

        let batch_id = "batch_completed_both";
        {
            // Insert the batch as Completed w/ both files
            let mut guard = mock_client.batches().write().unwrap();
            guard.insert(
                batch_id.to_string(),
                Batch {
                    output_file_id: Some("mock_out_id".to_string()),
                    error_file_id: Some("mock_err_id".to_string()),
                    id: batch_id.to_string(),
                    object: "batch".to_string(),
                    endpoint: "/v1/chat/completions".to_string(),
                    errors: None,
                    input_file_id: "some_input_file".to_string(),
                    completion_window: "24h".to_string(),
                    status: BatchStatus::Completed,
                    created_at: 0,
                    in_progress_at: None,
                    expires_at: None,
                    finalizing_at: None,
                    completed_at: None,
                    failed_at: None,
                    expired_at: None,
                    cancelling_at: None,
                    cancelled_at: None,
                    request_counts: None,
                    metadata: None,
                },
            );
        }

        // Add a file in the mock for each ID
        {
            let mut files_guard = mock_client.files().write().unwrap();
            files_guard.insert("mock_out_id".to_string(), Bytes::from("mock output data"));
            files_guard.insert("mock_err_id".to_string(), Bytes::from("mock error data"));
        }

        // Create local metadata
        let tmpdir = tempdir().unwrap();
        let metadata_path = tmpdir.path().join("metadata.json");
        let metadata = BatchMetadataBuilder::default()
            .batch_id(batch_id.to_string())
            .input_file_id("some_input_file".to_string())
            .output_file_id(Some("mock_out_id".to_string()))
            .error_file_id(Some("mock_err_id".to_string()))
            .build().unwrap();
        metadata.save_to_file(&metadata_path).await.unwrap();

        let mut triple = BatchFileTriple::new_for_test_with_metadata_path_unique(metadata_path.clone());
        triple.set_metadata_path(Some(metadata_path.clone()));

        let result = triple
            .check_for_and_download_output_and_error_online(&mock_client)
            .await;
        debug!("Result => {:?}", result);
        assert!(result.is_ok(), "Should succeed for completed batch with both files");

        // Confirm the downloaded contents
        let out_contents = std::fs::read_to_string(triple.effective_output_filename()).unwrap();
        let err_contents = std::fs::read_to_string(triple.effective_error_filename()).unwrap();
        pretty_assert_eq!(out_contents, "mock output data");
        pretty_assert_eq!(err_contents, "mock error data");
    }
}
