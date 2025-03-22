// ---------------- [ File: src/fresh_execute.rs ]
crate::ix!();

#[async_trait]
pub trait FreshExecute<Client,E> {
    type Success;
    async fn fresh_execute(&mut self, client: &Client) 
        -> Result<Self::Success, E>;
}

#[async_trait]
impl<C,E> FreshExecute<C,E> for BatchFileTriple
where 
    C: LanguageModelClientInterface<E>,

    // We no longer require “BatchDownloadError: From<E>” or “BatchProcessingError: From<E>”.
    // Instead, we do the normal “E: From<…>” for each error type that might bubble up:
    E
    : Debug 
    + Display 
    + From<BatchProcessingError>
    + From<BatchDownloadError>
    + From<JsonParseError>
    + From<std::io::Error>
    + From<OpenAIClientError>
    + From<BatchMetadataError>,
{
    type Success = BatchExecutionResult;

    async fn fresh_execute(&mut self, client: &C) -> Result<BatchExecutionResult, E> 
    {
        trace!("Inside fresh_execute for triple: {:?}", self);

        assert!(self.input().is_some());
        assert!(self.output().is_none());
        assert!(self.error().is_none());
        assert!(self.associated_metadata().is_none());

        info!("executing fresh batch processing for triple {:#?}", self);

        let input_filename    = self.effective_input_filename();
        let output_filename   = self.effective_output_filename();
        let error_filename    = self.effective_error_filename();
        let metadata_filename = self.effective_metadata_filename();

        info!("input_filename: {:?}",    input_filename);
        info!("output_filename: {:?}",   output_filename);
        info!("error_filename: {:?}",    error_filename);
        info!("metadata_filename: {:?}", metadata_filename);

        assert!(input_filename.exists());
        assert!(!output_filename.exists());
        assert!(!error_filename.exists());
        assert!(!metadata_filename.exists());

        // Upload file
        let input_file = client.upload_batch_file_path(&input_filename).await?;
        let input_file_id = input_file.id;

        // Create batch
        let batch = client.create_batch(&input_file_id).await?;
        let batch_id = batch.id.clone();

        // ** Save batch_id to metadata file **
        let mut metadata = BatchMetadata::with_input_id_and_batch_id(&input_file_id, &batch_id);
        metadata.save_to_file(&metadata_filename).await?;

        // Wait for completion
        let completed_batch = client.wait_for_batch_completion(&batch_id).await?;

        // Download output file
        let outputs = if let Some(output_file_id) = completed_batch.output_file_id {
            metadata.set_output_file_id(Some(output_file_id));
            metadata.save_to_file(&metadata_filename).await?;
            self.download_output_file(client).await?;
            let outputs = load_output_file(&output_filename).await?;
            Some(outputs)
        } else {
            None
        };

        // Handle errors if any
        let errors = if let Some(error_file_id) = completed_batch.error_file_id {
            metadata.set_error_file_id(Some(error_file_id));
            metadata.save_to_file(&metadata_filename).await?;
            self.download_error_file(client).await?;
            let errors = load_error_file(&error_filename).await?;
            Some(errors)
        } else {
            None
        };

        Ok(BatchExecutionResult::new(outputs, errors))
    }
}

#[cfg(test)]
mod fresh_execute_tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;
    use tracing::{debug, error, info, trace, warn};
    use futures::executor::block_on;

    /// A small helper that reproduces how the mock's create_batch
    /// forms its "batch_id" from the `file_path`.
    fn generate_mock_batch_id_for(input_file_path: &Path) -> String {
        // Exactly as the mock does:
        let input_file_id = format!("mock_file_id_{}", input_file_path.display());
        format!("mock_batch_id_for_{}", input_file_id)
    }

    /// For "immediate_failure" or "eventual_failure" we want the final batch to end up with status=Failed.
    /// Because the mock by default toggles from InProgress -> Completed, we can forcibly override
    /// the final result to be "Failed" on the second retrieval. We'll do that by storing the
    /// batch as "InProgress" initially with a custom key in `mock_batch_config`.
    fn configure_mock_batch_for_failure(
        mock_client: &MockLanguageModelClient<MockBatchClientError>,
        batch_id: &str,
        is_immediate: bool,
    ) {
        // If "immediate", just set it to Failed right away:
        if is_immediate {
            let mut guard = mock_client.batches().write().unwrap();
            guard.insert(
                batch_id.to_string(),
                Batch {
                    id: batch_id.to_string(),
                    object: "batch".to_string(),
                    endpoint: "/v1/chat/completions".to_string(),
                    errors: None,
                    input_file_id: format!("immediate_fail_for_{batch_id}"),
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
        } else {
            // "eventual_failure": start InProgress, then on second retrieval => fail
            // We'll store in mock_batch_config: fails_on_attempt_1 => set
            {
                let mut c = mock_client.mock_batch_config().write().unwrap();
                c.fails_on_attempt_1_mut().insert(batch_id.to_string());
            }
            // Also store an initial "InProgress" batch so we can see the toggling
            let mut guard = mock_client.batches().write().unwrap();
            guard.insert(
                batch_id.to_string(),
                Batch {
                    id: batch_id.to_string(),
                    object: "batch".to_string(),
                    endpoint: "/v1/chat/completions".to_string(),
                    errors: None,
                    input_file_id: format!("eventual_fail_for_{batch_id}"),
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
    }

    #[tracing::instrument(level = "trace", skip(mock_client))]
    pub fn configure_mock_batch_for_success(
        mock_client: &MockLanguageModelClient<MockBatchClientError>,
        batch_id: &str,
        want_output: bool,
        want_error: bool,
    ) {
        trace!("Configuring mock batch for success with batch_id='{}', want_output={}, want_error={}", batch_id, want_output, want_error);

        // Force the batch's status = Completed and set output_file_id/error_file_id if requested
        {
            let mut guard = mock_client.batches().write().unwrap();
            match guard.get_mut(batch_id) {
                Some(batch_entry) => {
                    debug!("Found existing batch entry for batch_id='{}'; setting status=Completed.", batch_id);
                    batch_entry.status = BatchStatus::Completed;
                    if want_output {
                        batch_entry.output_file_id = Some("mock_out_file_id".to_string());
                    }
                    if want_error {
                        batch_entry.error_file_id = Some("mock_err_file_id".to_string());
                    }
                }
                None => {
                    warn!("No existing batch entry for batch_id='{}'; inserting a new one with Completed status.", batch_id);
                    guard.insert(
                        batch_id.to_string(),
                        Batch {
                            id: batch_id.to_string(),
                            object: "batch".to_string(),
                            endpoint: "/v1/chat/completions".to_string(),
                            errors: None,
                            input_file_id: "inserted_dummy".to_string(),
                            completion_window: "24h".to_string(),
                            status: BatchStatus::Completed,
                            output_file_id: if want_output {
                                Some("mock_out_file_id".to_string())
                            } else {
                                None
                            },
                            error_file_id: if want_error {
                                Some("mock_err_file_id".to_string())
                            } else {
                                None
                            },
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
            }
        }

        // Insert the corresponding file contents into the mock's "files" map so
        // that calls to `file_content("mock_out_file_id")` or `file_content("mock_err_file_id")`
        // will return valid JSON that can be parsed as BatchResponseRecord.
        {
            let mut files_guard = mock_client.files().write().unwrap();

            if want_output {
                debug!("Inserting mock_out_file_id with a valid BatchResponseRecord JSON line.");
                files_guard.insert(
                    "mock_out_file_id".to_string(),
                    Bytes::from(
    r#"{
      "id": "batch_req_mock_output",
      "custom_id": "mock_out",
      "response": {
        "status_code": 200,
        "request_id": "resp_req_mock_output",
        "body": {
          "id": "success-id",
          "object": "chat.completion",
          "created": 0,
          "model": "test-model",
          "choices": [],
          "usage": {
            "prompt_tokens": 0,
            "completion_tokens": 0,
            "total_tokens": 0
          }
        }
      },
      "error": null
    }"#,
                    ),
                );
            }

            if want_error {
                debug!("Inserting mock_err_file_id with a valid BatchResponseRecord JSON line (status=400).");
                files_guard.insert(
                    "mock_err_file_id".to_string(),
                    Bytes::from(
    r#"{
      "id": "batch_req_mock_error",
      "custom_id": "mock_err",
      "response": {
        "status_code": 400,
        "request_id": "resp_req_mock_error",
        "body": {
          "error": {
            "message": "Some error message",
            "type": "test_error",
            "param": null,
            "code": null
          }
        }
      },
      "error": null
    }"#,
                    ),
                );
            }
        }

        trace!("configure_mock_batch_for_success done for batch_id='{}'", batch_id);
    }

    #[traced_test]
    async fn test_fresh_execute_success_error_only() {
        info!("Beginning test_fresh_execute_success_error_only");
        let workspace = BatchWorkspace::new_temp().await.expect("expected workspace construction success");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        debug!("Constructed mock client: {:?}", mock_client);

        // We'll create a local input file
        let tmp_dir = tempdir().expect("Failed to create temp dir");
        let input_file_path = tmp_dir.path().join("input.json");
        fs::write(&input_file_path, b"{}").unwrap();

        // Create the triple
        let mut triple = BatchFileTriple::new_for_test_with_in_out_err_paths(
            workspace,
            input_file_path.clone(),
            None,
            None,
        );

        // The real batch_id is "mock_batch_id_for_mock_file_id_{canonical_path}", 
        // so we do:
        let final_batch_id = generate_mock_batch_id_for(&input_file_path);

        // Here is the important fix:
        // We want the final to be Completed with *only* error_file_id => (false, true)
        mock_client.configure_inprogress_then_complete_with(&final_batch_id, false, true);

        // Now call fresh_execute
        let exec_result = triple.fresh_execute(&mock_client).await;
        debug!("Result from fresh_execute: {:?}", exec_result);

        assert!(exec_result.is_ok(), "Should succeed with error-only scenario");
        let result = exec_result.unwrap();
        assert!(result.outputs().is_none(), "No output data expected");
        assert!(result.errors().is_some(), "Should have error data");

        info!("test_fresh_execute_success_error_only passed");
    }

    #[traced_test]
    async fn test_fresh_execute_success_both_output_and_error() {
        let workspace = BatchWorkspace::new_temp().await.expect("expected workspace construction success");
        info!("Beginning test_fresh_execute_success_both_output_and_error");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        debug!("Mock client: {:?}", mock_client);

        // local input file
        let tmp_dir = tempdir().unwrap();
        let input_file_path = tmp_dir.path().join("input.json");
        fs::write(&input_file_path, b"{\"test\":\"data\"}").unwrap();

        let mut triple = BatchFileTriple::new_for_test_with_in_out_err_paths(
            workspace,
            input_file_path.clone(),
            None,
            None,
        );

        let final_batch_id = generate_mock_batch_id_for(&input_file_path);

        // We want the final to be Completed with *both* output and error => (true, true)
        mock_client.configure_inprogress_then_complete_with(&final_batch_id, true, true);

        info!("Calling fresh_execute for both output and error scenario");
        let exec_result = triple.fresh_execute(&mock_client).await;
        debug!("exec_result: {:?}", exec_result);

        assert!(exec_result.is_ok(), "Should succeed with both output and error");
        let exec_result = exec_result.unwrap();
        assert!(exec_result.outputs().is_some(),  "Expected output data");
        assert!(exec_result.errors().is_some(),   "Expected error data");

        // Confirm the local JSONL files actually got written:
        let out_file = triple.effective_output_filename();
        let err_file = triple.effective_error_filename();
        assert!(out_file.exists(), "Output file should exist on disk");
        assert!(err_file.exists(), "Error file should exist on disk");

        info!("test_fresh_execute_success_both_output_and_error passed");
    }

    #[traced_test]
    async fn test_fresh_execute_immediate_failure() {
        let workspace = BatchWorkspace::new_temp()
            .await
            .expect("expected workspace construction success");

        info!("Beginning test_fresh_execute_immediate_failure");

        // Build the mock client:
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();

        // Create an input file:
        let tmp_dir = tempdir().unwrap();
        let raw_input_path = tmp_dir.path().join("input.txt");
        fs::write(&raw_input_path, b"some input content").unwrap();

        // Canonicalize so the generated batch_id will match what the mock uses:
        let real_path = std::fs::canonicalize(&raw_input_path).unwrap();
        let final_batch_id = generate_mock_batch_id_for(&real_path);

        // Mark that batch as immediately Failed BEFORE we do fresh_execute:
        mock_client.configure_failure(&final_batch_id, /*is_immediate=*/true);

        // Now reference that same real_path in the triple:
        let mut triple = BatchFileTriple::new_for_test_with_in_out_err_paths(
            workspace,
            real_path.clone(),
            None,
            None,
        );

        // Because we forcibly set it to "Failed" above, fresh_execute should return an error:
        let result = triple.fresh_execute(&mock_client).await;
        debug!("Result from immediate_failure fresh_execute: {:?}", result);

        // We expect an error because the batch was "Failed" from the start
        assert!(
            result.is_err(),
            "fresh_execute should fail if the batch is immediately failed"
        );
        info!("test_fresh_execute_immediate_failure passed");
    }

    #[traced_test]
    async fn test_fresh_execute_eventual_failure() {
        info!("Beginning test_fresh_execute_eventual_failure");
        let workspace = BatchWorkspace::new_temp().await.expect("expected workspace construction success");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();

        let tmp_dir = tempdir().unwrap();
        let input_file_path = tmp_dir.path().join("input.txt");
        fs::write(&input_file_path, b"some input content").unwrap();

        let mut triple = BatchFileTriple::new_for_test_with_in_out_err_paths(
            workspace,
            input_file_path.clone(),
            None,
            None,
        );

        let final_batch_id = generate_mock_batch_id_for(&input_file_path);
        // Mark that batch as eventually failing on the second retrieval
        configure_mock_batch_for_failure(&mock_client, &final_batch_id, /*is_immediate=*/false);

        let result = triple.fresh_execute(&mock_client).await;
        debug!("Result from eventual_failure fresh_execute: {:?}", result);

        assert!(
            result.is_err(),
            "fresh_execute should eventually fail when batch toggles to Failed"
        );
        info!("test_fresh_execute_eventual_failure passed");
    }

    /// Instead of `catch_unwind`, we rely on `#[should_panic(expected)]` 
    #[tokio::test]
    #[should_panic(expected = "assertion failed: input_filename.exists()")]
    async fn test_fresh_execute_missing_input_file() {
        info!("Beginning test_fresh_execute_missing_input_file");
        let workspace = BatchWorkspace::new_temp().await.expect("expected workspace construction success");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();

        let tmp_dir = tempdir().unwrap();
        let input_file_path = tmp_dir.path().join("missing_file.json");
        // We do NOT actually create the file => it doesn't exist.

        let mut triple = BatchFileTriple::new_for_test_with_in_out_err_paths(
            workspace,
            input_file_path.clone(),
            None,
            None,
        );

        // The code in `fresh_execute` does `assert!(input_filename.exists())`.
        triple.fresh_execute(&mock_client).await.unwrap();
    }

    #[tokio::test]
    #[should_panic(expected = "assertion failed: !metadata_filename.exists()")]
    async fn test_fresh_execute_metadata_already_exists() {
        info!("Beginning test_fresh_execute_metadata_already_exists");
        let workspace = BatchWorkspace::new_temp().await.expect("expected workspace construction success");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();

        // We'll have a valid input file:
        let tmp_dir = tempdir().unwrap();
        let input_file_path = tmp_dir.path().join("input.json");
        fs::write(&input_file_path, b"{}").unwrap();

        // We'll create a triple referencing it:
        let mut triple = BatchFileTriple::new_for_test_with_in_out_err_paths(
            workspace,
            input_file_path.clone(),
            None,
            None,
        );

        // Then forcibly create the metadata file so it already exists:
        let meta_path = triple.effective_metadata_filename();
        fs::write(&meta_path, b"some old metadata content").unwrap();

        // Because fresh_execute asserts that the metadata file does NOT exist,
        // we expect a panic:
        triple.fresh_execute(&mock_client).await.unwrap();
    }

    #[traced_test]
    async fn test_fresh_execute_openai_error_on_upload() {
        info!("Beginning test_fresh_execute_openai_error_on_upload");
        let workspace = BatchWorkspace::new_temp().await.expect("expected workspace construction success");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .fail_on_file_create_openai_error(true) // forcibly cause an OpenAI error
            .build()
            .unwrap();

        let tmp_dir = tempdir().unwrap();
        let input_file_path = tmp_dir.path().join("input.json");
        fs::write(&input_file_path, b"[1,2,3]").unwrap();

        let mut triple = BatchFileTriple::new_for_test_with_in_out_err_paths(
            workspace,
            input_file_path.clone(),
            None,
            None,
        );

        // The first step is to upload the batch file => that triggers an OpenAI error
        let result = triple.fresh_execute(&mock_client).await;
        debug!("Result from fresh_execute with forced OpenAI upload error: {:?}", result);

        assert!(
            result.is_err(),
            "Should fail due to forced OpenAI error on upload"
        );
        info!("test_fresh_execute_openai_error_on_upload passed");
    }

    #[traced_test]
    async fn test_fresh_execute_io_error_on_upload() {
        info!("Beginning test_fresh_execute_io_error_on_upload");
        let workspace = BatchWorkspace::new_temp().await.expect("expected workspace construction success");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .fail_on_file_create_other_error(true) // forcibly cause an IO error
            .build()
            .unwrap();

        let tmp_dir = tempdir().unwrap();
        let input_file_path = tmp_dir.path().join("input.json");
        fs::write(&input_file_path, b"[4,5,6]").unwrap();

        let mut triple = BatchFileTriple::new_for_test_with_in_out_err_paths(
            workspace,
            input_file_path.clone(),
            None,
            None,
        );

        let result = triple.fresh_execute(&mock_client).await;
        debug!("Result from fresh_execute with forced IO error on upload: {:?}", result);

        assert!(
            result.is_err(),
            "Should fail due to forced I/O error on upload"
        );
        info!("test_fresh_execute_io_error_on_upload passed");
    }

    #[traced_test]
    async fn test_fresh_execute_success_output_only() {
        let workspace = BatchWorkspace::new_temp().await.unwrap();
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();

        // create local input file
        let tmp_dir = tempdir().unwrap();
        let input_path = tmp_dir.path().join("input.json");
        fs::write(&input_path, b"{\"test\":\"data\"}").unwrap();

        let mut triple = BatchFileTriple::new_for_test_with_in_out_err_paths(
            workspace,
            input_path.clone(),
            None,
            None,
        );

        // The mock batch_id is `mock_batch_id_for_mock_file_id_{input_path}`
        let final_batch_id = format!("mock_batch_id_for_mock_file_id_{}", input_path.display());

        // -> This sets an InProgress batch + sets planned_completions to (true, false).
        mock_client.configure_inprogress_then_complete_with(&final_batch_id, true, false);

        // Now do fresh_execute => we expect outputs but no errors
        let exec_result = triple.fresh_execute(&mock_client).await;
        assert!(exec_result.is_ok());
        let exec_result = exec_result.unwrap();
        assert!(exec_result.outputs().is_some(), "Should have output data");
        assert!(exec_result.errors().is_none(), "Should have no error data");
    }
}
