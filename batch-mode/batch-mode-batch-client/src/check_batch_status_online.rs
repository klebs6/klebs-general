// ---------------- [ File: src/check_batch_status_online.rs ]
crate::ix!();

#[async_trait]
impl<E> CheckBatchStatusOnline<E> for BatchFileTriple
where
    E: From<BatchDownloadError>
        + From<OpenAIClientError>
        + From<BatchMetadataError>
        + From<std::io::Error>
        + Debug,
{
    async fn check_batch_status_online(
        &self,
        client: &dyn LanguageModelClientInterface<E>,
    ) -> Result<BatchOnlineStatus, E> {
        info!("checking batch status online");

        // Pick the correct metadata path if we have associated_metadata:
        let metadata_filename: PathBuf = if let Some(path) = self.associated_metadata() {
            path.clone()
        } else {
            self.effective_metadata_filename()
        };
        debug!("Using metadata file: {:?}", metadata_filename);

        let mut metadata = BatchMetadata::load_from_file(&metadata_filename).await?;
        let batch_id = metadata.batch_id().to_string();

        let batch = client.retrieve_batch(&batch_id).await?;
        match batch.status {
            BatchStatus::Completed => {
                // Only if completed do we store these IDs into the metadata.
                metadata.set_output_file_id(batch.output_file_id.clone());
                metadata.set_error_file_id(batch.error_file_id.clone());
                metadata.save_to_file(&metadata_filename).await?;

                Ok(BatchOnlineStatus::from(&batch))
            }
            BatchStatus::Failed => {
                Err(BatchDownloadError::BatchFailed { batch_id }.into())
            }
            BatchStatus::Validating
            | BatchStatus::InProgress
            | BatchStatus::Finalizing => {
                Err(BatchDownloadError::BatchStillProcessing { batch_id }.into())
            }
            _ => {
                Err(BatchDownloadError::UnknownBatchStatus {
                    batch_id,
                    status: batch.status.clone(),
                }
                .into())
            }
        }
    }
}

#[cfg(test)]
mod check_batch_status_online_tests {
    use super::*;
    use futures::executor::block_on;
    use tempfile::tempdir;
    use tracing::{debug, error, info, trace, warn};
    use std::fs;

    #[traced_test]
    async fn test_batch_completed_no_files() {
        info!("Starting test_batch_completed_no_files");
        trace!("Constructing a mock client...");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        debug!("Mock client: {:?}", mock_client);

        let batch_id = "test_batch_completed_no_files";
        trace!("Inserting batch with ID={}", batch_id);
        {
            let mut guard = mock_client.batches().write().unwrap();
            guard.insert(
                batch_id.to_string(),
                Batch {
                    id: batch_id.to_string(),
                    object: "batch".to_string(),
                    endpoint: "/v1/chat/completions".to_string(),
                    errors: None,
                    input_file_id: "input_file_id".to_string(),
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

        let tmpdir = tempdir().unwrap();
        let metadata_path = tmpdir.path().join("metadata.json");
        let metadata = BatchMetadataBuilder::default()
            .batch_id(batch_id.to_string())
            .input_file_id("input_file_id".to_string())
            .output_file_id(None)
            .error_file_id(None)
            .build()
            .unwrap();
        info!("Saving metadata at {:?}", metadata_path);
        metadata.save_to_file(&metadata_path).await.unwrap();

        trace!("Creating BatchFileTriple with known metadata path...");
        let mut triple = BatchFileTriple::new_for_test_with_metadata_path(metadata_path.clone());
        // Must be mutable so we can call set_metadata_path
        triple.set_metadata_path(Some(metadata_path.clone()));

        trace!("Calling check_batch_status_online...");
        let result = triple.check_batch_status_online(&mock_client).await;
        debug!("Result from check_batch_status_online: {:?}", result);

        assert!(
            result.is_ok(),
            "Should return Ok(...) for a completed batch with no output/error"
        );
        let online_status = result.unwrap();
        pretty_assert_eq!(online_status.output_file_available(), false);
        pretty_assert_eq!(online_status.error_file_available(), false);
        info!("test_batch_completed_no_files passed successfully.");
    }

    #[traced_test]
    async fn test_batch_completed_with_output_only() {
        info!("Starting test_batch_completed_with_output_only");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        debug!("Mock client: {:?}", mock_client);

        let batch_id = "test_batch_completed_with_output_only";
        {
            let mut guard = mock_client.batches().write().unwrap();
            guard.insert(
                batch_id.to_string(),
                Batch {
                    id: batch_id.to_string(),
                    object: "batch".to_string(),
                    endpoint: "/v1/chat/completions".to_string(),
                    errors: None,
                    input_file_id: "input_file_id".to_string(),
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

        let tmpdir = tempdir().unwrap();
        let metadata_path = tmpdir.path().join("metadata.json");
        let metadata = BatchMetadataBuilder::default()
            .batch_id(batch_id.to_string())
            .input_file_id("input_file_id".to_string())
            .output_file_id(Some("mock_output_file_id".to_string()))
            .error_file_id(None)
            .build()
            .unwrap();
        info!("Saving metadata at {:?}", metadata_path);
        metadata.save_to_file(&metadata_path).await.unwrap();

        let mut triple = BatchFileTriple::new_for_test_with_metadata_path(metadata_path.clone());
        triple.set_metadata_path(Some(metadata_path.clone()));

        let result = triple.check_batch_status_online(&mock_client).await;
        debug!("Result from check_batch_status_online: {:?}", result);

        assert!(
            result.is_ok(),
            "Should return Ok(...) for a completed batch with output only"
        );
        let online_status = result.unwrap();
        pretty_assert_eq!(online_status.output_file_available(), true);
        pretty_assert_eq!(online_status.error_file_available(), false);
        info!("test_batch_completed_with_output_only passed successfully.");
    }

    #[traced_test]
    async fn test_batch_completed_with_error_only() {
        info!("Starting test_batch_completed_with_error_only");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        debug!("Mock client: {:?}", mock_client);

        let batch_id = "test_batch_completed_with_error_only";
        {
            let mut guard = mock_client.batches().write().unwrap();
            guard.insert(
                batch_id.to_string(),
                Batch {
                    id: batch_id.to_string(),
                    object: "batch".to_string(),
                    endpoint: "/v1/chat/completions".to_string(),
                    errors: None,
                    input_file_id: "input_file_id".to_string(),
                    completion_window: "24h".to_string(),
                    status: BatchStatus::Completed,
                    output_file_id: None,
                    error_file_id: Some("mock_err_file_id".to_string()),
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

        let tmpdir = tempdir().unwrap();
        let metadata_path = tmpdir.path().join("metadata.json");
        let metadata = BatchMetadataBuilder::default()
            .batch_id(batch_id.to_string())
            .input_file_id("input_file_id".to_string())
            .output_file_id(None)
            .error_file_id(Some("mock_err_file_id".to_string()))
            .build()
            .unwrap();
        info!("Saving metadata at {:?}", metadata_path);
        metadata.save_to_file(&metadata_path).await.unwrap();

        let mut triple = BatchFileTriple::new_for_test_with_metadata_path(metadata_path.clone());
        triple.set_metadata_path(Some(metadata_path.clone()));

        let result = triple.check_batch_status_online(&mock_client).await;
        debug!("Result from check_batch_status_online: {:?}", result);

        assert!(
            result.is_ok(),
            "Should return Ok(...) for a completed batch with error only"
        );
        let online_status = result.unwrap();
        pretty_assert_eq!(online_status.output_file_available(), false);
        pretty_assert_eq!(online_status.error_file_available(), true);
        info!("test_batch_completed_with_error_only passed successfully.");
    }

    #[traced_test]
    async fn test_batch_completed_with_output_and_error() {
        info!("Starting test_batch_completed_with_output_and_error");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        debug!("Mock client: {:?}", mock_client);

        let batch_id = "test_batch_completed_with_output_and_error";
        {
            let mut guard = mock_client.batches().write().unwrap();
            guard.insert(
                batch_id.to_string(),
                Batch {
                    id: batch_id.to_string(),
                    object: "batch".to_string(),
                    endpoint: "/v1/chat/completions".to_string(),
                    errors: None,
                    input_file_id: "input_file_id".to_string(),
                    completion_window: "24h".to_string(),
                    status: BatchStatus::Completed,
                    output_file_id: Some("mock_output_file_id".to_string()),
                    error_file_id: Some("mock_err_file_id".to_string()),
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

        let tmpdir = tempdir().unwrap();
        let metadata_path = tmpdir.path().join("metadata.json");
        let metadata = BatchMetadataBuilder::default()
            .batch_id(batch_id.to_string())
            .input_file_id("input_file_id".to_string())
            .output_file_id(Some("mock_output_file_id".to_string()))
            .error_file_id(Some("mock_err_file_id".to_string()))
            .build()
            .unwrap();
        info!("Saving metadata at {:?}", metadata_path);
        metadata.save_to_file(&metadata_path).await.unwrap();

        let mut triple = BatchFileTriple::new_for_test_with_metadata_path(metadata_path.clone());
        triple.set_metadata_path(Some(metadata_path.clone()));

        let result = triple.check_batch_status_online(&mock_client).await;
        debug!("Result from check_batch_status_online: {:?}", result);

        assert!(
            result.is_ok(),
            "Should return Ok(...) for a completed batch with both output and error"
        );
        let online_status = result.unwrap();
        pretty_assert_eq!(online_status.output_file_available(), true);
        pretty_assert_eq!(online_status.error_file_available(), true);
        info!("test_batch_completed_with_output_and_error passed successfully.");
    }

    #[traced_test]
    async fn test_batch_failed() {
        info!("Starting test_batch_failed");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        debug!("Mock client: {:?}", mock_client);

        let batch_id = "test_batch_failed";
        {
            let mut guard = mock_client.batches().write().unwrap();
            guard.insert(
                batch_id.to_string(),
                Batch {
                    id: batch_id.to_string(),
                    object: "batch".to_string(),
                    endpoint: "/v1/chat/completions".to_string(),
                    errors: None,
                    input_file_id: "input_file_id".to_string(),
                    completion_window: "24h".to_string(),
                    status: BatchStatus::Failed,
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

        let tmpdir = tempdir().unwrap();
        let metadata_path = tmpdir.path().join("metadata.json");
        let metadata = BatchMetadataBuilder::default()
            .batch_id(batch_id.to_string())
            .input_file_id("input_file_id".to_string())
            .build()
            .unwrap();
        metadata.save_to_file(&metadata_path).await.unwrap();

        let mut triple = BatchFileTriple::new_for_test_with_metadata_path(metadata_path.clone());
        triple.set_metadata_path(Some(metadata_path.clone()));

        let result = triple.check_batch_status_online(&mock_client).await;
        debug!("Result from check_batch_status_online: {:?}", result);

        assert!(result.is_err(), "Should return Err(...) for a failed batch");
        info!("test_batch_failed passed successfully.");
    }

    #[traced_test]
    async fn test_batch_inprogress() {
        info!("Starting test_batch_inprogress");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        debug!("Mock client: {:?}", mock_client);

        let batch_id = "test_batch_inprogress";
        {
            let mut guard = mock_client.batches().write().unwrap();
            guard.insert(
                batch_id.to_string(),
                Batch {
                    id: batch_id.to_string(),
                    object: "batch".to_string(),
                    endpoint: "/v1/chat/completions".to_string(),
                    errors: None,
                    input_file_id: "input_file_id".to_string(),
                    completion_window: "24h".to_string(),
                    status: BatchStatus::InProgress,
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

        let tmpdir = tempdir().unwrap();
        let metadata_path = tmpdir.path().join("metadata.json");
        let metadata = BatchMetadataBuilder::default()
            .batch_id(batch_id.to_string())
            .input_file_id("input_file_id".to_string())
            .build()
            .unwrap();
        metadata.save_to_file(&metadata_path).await.unwrap();

        let mut triple = BatchFileTriple::new_for_test_with_metadata_path(metadata_path.clone());
        triple.set_metadata_path(Some(metadata_path.clone()));

        let result = triple.check_batch_status_online(&mock_client).await;
        debug!("Result from check_batch_status_online: {:?}", result);

        assert!(
            result.is_err(),
            "Should return Err(...) for an in-progress batch"
        );
        info!("test_batch_inprogress passed successfully.");
    }

    #[traced_test]
    async fn test_batch_unknown_status() {
        info!("Starting test_batch_unknown_status");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        debug!("Mock client: {:?}", mock_client);

        let batch_id = "test_batch_unknown_status";
        {
            let mut guard = mock_client.batches().write().unwrap();
            let mut some_batch = Batch {
                id: batch_id.to_string(),
                object: "batch".to_string(),
                endpoint: "/v1/chat/completions".to_string(),
                errors: None,
                input_file_id: "input_file_id".to_string(),
                completion_window: "24h".to_string(),
                status: BatchStatus::InProgress,
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
            };
            // Force an unknown or "invalid" status:
            some_batch.status = BatchStatus::Cancelled;
            guard.insert(batch_id.to_string(), some_batch);
        }

        let tmpdir = tempdir().unwrap();
        let metadata_path = tmpdir.path().join("metadata.json");
        let metadata = BatchMetadataBuilder::default()
            .batch_id(batch_id.to_string())
            .input_file_id("input_file_id".to_string())
            .build()
            .unwrap();
        metadata.save_to_file(&metadata_path).await.unwrap();

        let mut triple = BatchFileTriple::new_for_test_with_metadata_path(metadata_path.clone());
        triple.set_metadata_path(Some(metadata_path.clone()));

        let result = triple.check_batch_status_online(&mock_client).await;
        debug!("Result from check_batch_status_online: {:?}", result);

        assert!(
            result.is_err(),
            "Should return Err(...) for an unknown batch status"
        );
        info!("test_batch_unknown_status passed successfully.");
    }
}
