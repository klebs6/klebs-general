crate::ix!();

/// A simple error type for the mock, so we can unify all the error conversions.
error_tree!{
    pub enum MockBatchClientError {
        OpenAIClientError(OpenAIClientError),
        BatchDownloadError(BatchDownloadError),
        BatchMetadataError(BatchMetadataError),
        IoError(std::io::Error),

        /// Required so that `E: From<BatchProcessingError>` is satisfied.
        BatchProcessingError,

        /// Required so that `E: From<JsonParseError>` is satisfied.
        JsonParseError(JsonParseError),
    }
}

#[derive(Getters, Setters, Builder, Debug)]
#[builder(pattern = "owned")]
pub struct MockLanguageModelClient<E> {
    #[getset(get = "pub", set = "pub")]
    #[builder(default)]
    batches: RwLock<HashMap<String, Batch>>,

    #[getset(get = "pub", set = "pub")]
    #[builder(default)]
    files: RwLock<HashMap<String, Bytes>>,

    #[builder(default="false")]
    #[getset(get = "pub", set = "pub")]
    fail_on_file_create_openai_error: bool,

    #[builder(default="false")]
    #[getset(get = "pub", set = "pub")]
    fail_on_file_create_other_error: bool,

    #[builder(default)]
    _error_marker: PhantomData<E>,

    /// NEW FIELD: stores extra toggles and attempt counters for advanced mock behaviors
    #[getset(get = "pub", set = "pub")]
    #[builder(default)]
    mock_batch_config: RwLock<MockBatchConfig>,
}

#[derive(MutGetters,Getters,Setters,Builder,Debug,Default)]
#[builder(setter(into), default, pattern = "owned")]
#[getset(get="pub",set="pub",get_mut="pub")]
pub struct MockBatchConfig {
    /// If a batch_id is inserted here, the mock flips it from InProgress->Failed on the first retrieval.
    fails_on_attempt_1: HashSet<String>,

    /// Tracks how many times we've retrieved a given batch_id.
    attempt_counters: HashMap<String, u32>,

    /// For a batch_id, we store (want_output, want_error). On retrieval 
    /// if we flip from InProgress->Completed, we set those file IDs.
    #[builder(default)]
    planned_completions: HashMap<String, (bool, bool)>,
}

impl<E> MockLanguageModelClient<E>
where
    E: From<OpenAIClientError> + From<std::io::Error> + Debug + Send + Sync,
{
    /// Configure the mock so that the next time we retrieve `batch_id`, it will show
    /// status = InProgress, then on retrieve => flip to Completed with the specified
    /// (want_output, want_error).
    ///
    /// This is used by “success” scenarios where we want the batch to end up Completed
    /// with optional output_file_id and/or error_file_id.
    pub fn configure_inprogress_then_complete_with(
        &self,
        batch_id: &str,
        want_output: bool,
        want_error: bool,
    ) {
        // 1) Insert an InProgress batch in `batches`
        {
            let mut map_guard = self.batches().write().unwrap();
            map_guard.insert(
                batch_id.to_string(),
                Batch {
                    id: batch_id.to_string(),
                    object: "batch".to_string(),
                    endpoint: "/v1/chat/completions".to_string(),
                    errors: None,
                    input_file_id: batch_id.to_string(),
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

        // 2) In `mock_batch_config`, set `planned_completions[batch_id] = (want_output, want_error)`
        {
            let mut cfg_guard = self.mock_batch_config().write().unwrap();
            cfg_guard
                .planned_completions_mut()
                .insert(batch_id.to_string(), (want_output, want_error));
        }
    }

    pub fn configure_failure(&self, batch_id: &str, is_immediate: bool) {
        let mut guard = self.batches().write().unwrap();
        if is_immediate {
            // If the user wants an immediate failure, directly insert or overwrite
            // the batch with status = Failed. That ensures retrieve_batch sees it.
            guard.insert(
                batch_id.to_string(),
                Batch {
                    id: batch_id.to_string(),
                    object: "batch".to_string(),
                    endpoint: "/v1/chat/completions".to_string(),
                    errors: None,
                    input_file_id: format!("immediate_fail_for_{}", batch_id),
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
            // For an eventual failure, we store InProgress plus note that the
            // first retrieve will set status=Failed:
            let mut cfg = self.mock_batch_config().write().unwrap();
            cfg.fails_on_attempt_1_mut().insert(batch_id.to_string());
            drop(cfg);

            guard.insert(
                batch_id.to_string(),
                Batch {
                    id: batch_id.to_string(),
                    object: "batch".to_string(),
                    endpoint: "/v1/chat/completions".to_string(),
                    errors: None,
                    input_file_id: format!("eventual_fail_for_{}", batch_id),
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
}

impl<E> MockLanguageModelClient<E>
where
    E: From<OpenAIClientError>
        + From<std::io::Error>
        + Debug
        + Send
        + Sync,
{
    pub fn new() -> Self {
        trace!("Constructing MockLanguageModelClient (checking OPENAI_API_KEY)...");
        let _ = std::env::var("OPENAI_API_KEY")
            .expect("OPENAI_API_KEY environment variable not set (Mock client requires it for test)");

        MockLanguageModelClientBuilder::<E>::default()
            .build()
            .unwrap()
    }

    /// Helper to forcibly mark the given `batch_id` as "InProgress" in the 
    /// `batches` map with a note in our `mock_batch_config` so that 
    /// on the *very next retrieve*, we flip it to "Completed" *with* 
    /// an `output_file_id` and/or `error_file_id`.
    ///
    /// This ensures that when `wait_for_batch_completion` sees it become
    /// "Completed," it already has the relevant file IDs.
    pub fn set_batch_to_inprogress_then_complete_with(
        &self,
        batch_id: &str,
        want_output: bool,
        want_error: bool,
    ) {
        {
            let mut guard = self.batches().write().unwrap();
            guard.insert(
                batch_id.to_string(),
                Batch {
                    id: batch_id.to_string(),
                    object: "batch".to_string(),
                    endpoint: "/v1/chat/completions".to_string(),
                    errors: None,
                    input_file_id: format!("some_input_file_for_{}", batch_id),
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

        // Also place the "final" file IDs into mock_batch_config so that
        // on next retrieval we can flip to Completed and set them:
        let mut config_guard = self.mock_batch_config().write().unwrap();
        config_guard.attempt_counters.insert(batch_id.to_string(), 0);

        // We'll store a small 'BatchOutcome' struct or booleans:
        config_guard
            .planned_completions
            .insert(batch_id.to_string(), (want_output, want_error));
    }
}

#[async_trait]
impl<E> RetrieveBatchById for MockLanguageModelClient<E>
where
    E: From<OpenAIClientError>
        + From<std::io::Error>
        + Debug
        + Send
        + Sync,
{
    type Error = E;

    async fn retrieve_batch(&self, batch_id: &str) -> Result<Batch, Self::Error> {
        info!("Mock: retrieve_batch called with batch_id={batch_id}");

        // Quick checks for forced errors:
        if batch_id.is_empty() {
            let openai_err = OpenAIClientError::ApiError(OpenAIApiError {
                message: "Cannot retrieve batch with empty batch_id".to_owned(),
                r#type: None,
                param: None,
                code: None,
            });
            return Err(E::from(openai_err));
        }
        if batch_id == "trigger_api_error" {
            let openai_err = OpenAIClientError::ApiError(OpenAIApiError {
                message: "Simulated retrieve_batch OpenAI error".to_string(),
                r#type: None,
                param: None,
                code: None,
            });
            return Err(E::from(openai_err));
        }
        if batch_id == "trigger_other_error" {
            let io_err = std::io::Error::new(
                std::io::ErrorKind::Other,
                "Simulated retrieve_batch non-OpenAI error",
            );
            return Err(E::from(io_err));
        }

        // First, grab the mock_batch_config write-guard, extract all needed info,
        // and store it in local variables. Then drop the guard so we don't run
        // into multiple borrow conflicts later when we continue logic.
        let (attempts_so_far, is_fail_on_attempt_1, planned) = {
            let mut cfg_guard = self.mock_batch_config().write().unwrap();

            // 1) Bump attempt count or initialize to 1
            let attempts_ref = cfg_guard
                .attempt_counters_mut()
                .entry(batch_id.to_string())
                .and_modify(|count| *count += 1)
                .or_insert(1);
            let attempts_value = *attempts_ref;

            // 2) Check if this batch_id is in fails_on_attempt_1
            let fail_1 = cfg_guard.fails_on_attempt_1().contains(batch_id);

            // 3) Check if there's a (want_output, want_error) plan
            let plan = cfg_guard
                .planned_completions()
                .get(batch_id)
                .cloned(); // Option<(bool, bool)>

            (attempts_value, fail_1, plan)
        };

        // Now handle the "batches" map, storing or retrieving the entry:
        let mut map_guard = self.batches().write().unwrap();
        let batch_entry = map_guard.entry(batch_id.to_string()).or_insert_with(|| {
            info!("Mock: auto-creating an InProgress batch for id={batch_id}");
            Batch {
                id: batch_id.to_string(),
                object: "batch".to_string(),
                endpoint: "/v1/chat/completions".to_string(),
                errors: None,
                input_file_id: format!("auto_{batch_id}"),
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
            }
        });

        // If the user wants an "immediate_failure_id", forcibly set to Failed:
        if batch_id == "immediate_failure_id" {
            batch_entry.status = BatchStatus::Failed;
        }

        // If it's in fails_on_attempt_1, and attempts_so_far == 1 => fail now
        if is_fail_on_attempt_1 && attempts_so_far == 1 {
            info!("Mock: fails_on_attempt_1 => flipping to Failed on attempt=1");
            batch_entry.status = BatchStatus::Failed;
        }

        // If we're still InProgress => flip to Completed
        // Then apply the planned (want_output, want_error), if any
        if batch_entry.status == BatchStatus::InProgress {
            info!("Mock: flipping {batch_id} from InProgress to Completed now");
            batch_entry.status = BatchStatus::Completed;

            if let Some((want_output, want_error)) = planned {
                if want_output {
                    batch_entry.output_file_id = Some("mock_out_file_id".to_owned());
                    let mut files_guard = self.files().write().unwrap();
                    files_guard.insert(
                        "mock_out_file_id".to_owned(),
                        Bytes::from("{\"mock\":\"output data\"}"),
                    );
                }
                if want_error {
                    batch_entry.error_file_id = Some("mock_err_file_id".to_owned());
                    let mut files_guard = self.files().write().unwrap();
                    files_guard.insert(
                        "mock_err_file_id".to_owned(),
                        Bytes::from("{\"mock\":\"error data\"}"),
                    );
                }
            }
        }

        let final_batch = batch_entry.clone();
        drop(map_guard);

        debug!(
            "Mock: retrieve_batch => returning final batch with status={:?}",
            final_batch.status
        );
        Ok(final_batch)
    }
}

#[async_trait]
impl<E> GetBatchFileContent for MockLanguageModelClient<E>
where
    E: From<OpenAIClientError>
        + From<std::io::Error>
        + Debug
        + Send
        + Sync,
{
    type Error = E;

    async fn file_content(&self, file_id: &str) -> Result<Bytes, Self::Error> {
        info!("Mock: file_content called with file_id={}", file_id);

        // NEW: If the user calls "valid_file_id" and we don't have it, we insert it:
        {
            let mut guard = self.files().write().unwrap();
            if file_id == "valid_file_id" && !guard.contains_key(file_id) {
                debug!("Mock: auto-inserting 'valid_file_id' => 'some mock content'");
                guard.insert("valid_file_id".to_string(), Bytes::from("some mock content"));
            }
        }

        let files_guard = self.files().read().unwrap();
        if let Some(data) = files_guard.get(file_id) {
            debug!("Mock: Found file content for id={}", file_id);
            Ok(data.clone())
        } else {
            warn!("Mock: No file found for id={}", file_id);
            let openai_err = OpenAIClientError::ApiError(OpenAIApiError {
                message: format!("No file found for id={}", file_id),
                r#type: None,
                param: None,
                code: None,
            });
            Err(E::from(openai_err))
        }
    }
}

#[async_trait]
impl<E> CreateBatch for MockLanguageModelClient<E>
where
    E: From<OpenAIClientError>
        + From<std::io::Error>
        + Debug
        + Send
        + Sync,
{
    type Error = E;

    async fn create_batch(&self, input_file_id: &str) -> Result<Batch, Self::Error> {
        info!("Mock: create_batch called with input_file_id={}", input_file_id);

        // Basic forced-error checks:
        if input_file_id.is_empty() {
            let openai_err = OpenAIClientError::ApiError(OpenAIApiError {
                message: "Cannot create batch with empty input_file_id".to_string(),
                r#type: None,
                param: None,
                code: None,
            });
            return Err(E::from(openai_err));
        }
        if input_file_id == "trigger_api_error" {
            let openai_err = OpenAIClientError::ApiError(OpenAIApiError {
                message: "Simulated OpenAI error (trigger_api_error)".to_string(),
                r#type: None,
                param: None,
                code: None,
            });
            return Err(E::from(openai_err));
        }
        if input_file_id == "trigger_other_error" {
            let io_err = std::io::Error::new(std::io::ErrorKind::Other, "Simulated other error");
            return Err(E::from(io_err));
        }

        // Normal success path with "InProgress" unless there's already a special batch:
        let mock_id = format!("mock_batch_id_for_{}", input_file_id);

        let mut map_guard = self.batches().write().unwrap();
        if let Some(existing) = map_guard.get(&mock_id) {
            // If there's already a batch (e.g. set to Failed by configure_failure),
            // just return it so we don't overwrite:
            return Ok(existing.clone());
        }

        // Otherwise insert a new InProgress:
        let new_batch = Batch {
            id: mock_id.clone(),
            object: "batch".to_string(),
            endpoint: "/v1/chat/completions".to_string(),
            errors: None,
            input_file_id: input_file_id.to_string(),
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
        map_guard.insert(mock_id.clone(), new_batch.clone());
        Ok(new_batch)
    }
}

#[async_trait]
impl<E> WaitForBatchCompletion for MockLanguageModelClient<E>
where
    E: From<OpenAIClientError>
        + From<std::io::Error>
        + Debug
        + Send
        + Sync,
{
    type Error = E;

    async fn wait_for_batch_completion(&self, batch_id: &str) -> Result<Batch, Self::Error> {
        info!("Mock: wait_for_batch_completion called with batch_id={}", batch_id);

        for attempt in 0..3 {
            debug!("Mock: attempt #{} checking batch_id={}", attempt, batch_id);

            let batch = self.retrieve_batch(batch_id).await?;
            match batch.status {
                BatchStatus::Completed => {
                    debug!("Mock: batch is Completed => returning Ok(batch)");
                    return Ok(batch);
                }
                BatchStatus::Failed => {
                    warn!("Mock: batch is Failed => returning error");
                    let openai_err = OpenAIClientError::ApiError(OpenAIApiError {
                        message: "Batch failed".to_owned(),
                        r#type: None,
                        param: None,
                        code: None,
                    });
                    return Err(E::from(openai_err));
                }
                // If it's not Completed or Failed, we just wait & try again:
                _ => {
                    info!("Mock: batch has status={:?}, continuing loop", batch.status);
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }

        // If 3 tries didn’t produce Completed or Failed, we treat it as a timeout
        let openai_err = OpenAIClientError::ApiError(OpenAIApiError {
            message: format!("Timed out waiting for batch {batch_id} to complete"),
            r#type: None,
            param: None,
            code: None,
        });
        Err(E::from(openai_err))
    }
}

#[async_trait]
impl<E> UploadBatchFileCore for MockLanguageModelClient<E>
where
    E: From<OpenAIClientError>
        + From<std::io::Error>
        + Debug
        + Send
        + Sync,
{
    type Error = E;

    async fn upload_batch_file_path(
        &self,
        file_path: &Path
    ) -> Result<OpenAIFile, Self::Error> {
        info!("Mock: upload_batch_file_path called with path={:?}", file_path);

        let path_str = file_path.display().to_string();

        // If user specifically wants an OpenAI error scenario:
        if path_str.contains("trigger_api_error") {
            warn!("Mock: forcibly returning an OpenAIClientError for file upload (trigger_api_error detected)");
            let openai_err = OpenAIClientError::ApiError(OpenAIApiError {
                message: "Simulated upload error (mocked as openai error)".to_string(),
                r#type: None,
                param: None,
                code: None,
            });
            return Err(E::from(openai_err));
        }

        // If user specifically wants a non-OpenAI IO error scenario:
        if path_str.contains("trigger_other_error") {
            warn!("Mock: forcibly returning an IoError for file upload (trigger_other_error detected)");
            let io_err = std::io::Error::new(
                std::io::ErrorKind::Other,
                "Simulated other error triggered in upload_batch_file_path"
            );
            return Err(E::from(io_err));
        }

        // If user sets fail_on_file_create_openai_error => simulate an OpenAI error
        if *self.fail_on_file_create_openai_error() {
            warn!("Mock: forcibly returning an OpenAIClientError for file upload due to fail_on_file_create_openai_error=true");
            let openai_err = OpenAIClientError::ApiError(OpenAIApiError {
                message: "Simulated upload error (mocked as openai error)".to_string(),
                r#type: None,
                param: None,
                code: None,
            });
            return Err(E::from(openai_err));
        }

        // If user sets fail_on_file_create_other_error => simulate an I/O error
        if *self.fail_on_file_create_other_error() {
            warn!("Mock: forcibly returning an IoError for file upload due to fail_on_file_create_other_error=true");
            let io_err = std::io::Error::new(
                std::io::ErrorKind::Other,
                "Simulated other error triggered in upload_batch_file_path"
            );
            return Err(E::from(io_err));
        }

        // Normal path: check if the file physically exists
        if !file_path.exists() {
            let io_err = std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found at {:?}", file_path),
            );
            error!("Mock: returning IoError for missing file {:?}", file_path);
            return Err(E::from(io_err));
        }

        // If we get here, success
        let file_id = format!("mock_file_id_{}", path_str);
        debug!("Mock: Storing synthetic file content for file_id={}", file_id);

        {
            let mut files_guard = self.files().write().unwrap();
            files_guard.insert(file_id.clone(), Bytes::from("mock file content"));
        }

        let openai_file = OpenAIFile {
            id: file_id.clone(),
            bytes: 123,
            created_at: 0,
            filename: file_path
                .file_name()
                .map(|os| os.to_string_lossy().into_owned())
                .unwrap_or_else(|| "unknown".to_string()),
            purpose: OpenAIFilePurpose::Batch,
            object: "file".to_string(),
            status: Some("uploaded".to_string()),
            status_details: None,
        };

        Ok(openai_file)
    }
}


// Finally, implement the aggregator trait itself:
#[async_trait]
impl<E> LanguageModelClientInterface<E> for MockLanguageModelClient<E>
where
    E: From<OpenAIClientError>
        + From<BatchDownloadError>
        + From<std::io::Error>
        + From<BatchMetadataError>
        + Debug
        + Send
        + Sync,
{
    // No additional methods; aggregator trait is just sub-traits.
}

// Demonstration of how you might test with this mock:
// (We strongly prefer #[traced_test] alone; we wrap async with block_on.)
use futures::executor::block_on;


#[cfg(test)]
mod mock_client_handle_tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::tempdir;
    use tracing::{debug, error, info, trace, warn};

    #[traced_test]
    async fn test_mock_language_model_client_basic_usage() {
        info!("Starting test_mock_language_model_client_basic_usage");

        let mock = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .expect("Failed to build mock client");

        info!("Creating a batch via the mock client...");
        let created = mock.create_batch("example_file_id").await;
        assert!(created.is_ok(), "Should create batch successfully");
        let created_batch = created.unwrap();
        pretty_assert_eq!(
            created_batch.status,
            BatchStatus::InProgress,
            "Newly created mock batch should be InProgress"
        );

        info!("Retrieving the newly created batch...");
        let retrieved = mock.retrieve_batch(&created_batch.id).await;
        assert!(retrieved.is_ok(), "Should retrieve batch successfully");
        let retrieved_batch = retrieved.unwrap();
        pretty_assert_eq!(
            retrieved_batch.id, created_batch.id,
            "Retrieved batch ID should match"
        );

        info!("Waiting for batch completion...");
        let wait_result = mock.wait_for_batch_completion(&created_batch.id).await;
        debug!("Result from wait_for_batch_completion: {:?}", wait_result);

        // Because "mock_batch_id_for_example_file_id" now toggles to Completed after the second retrieve,
        // we expect success:
        assert!(wait_result.is_ok(), "Should complete batch successfully");
        let completed_batch = wait_result.unwrap();
        pretty_assert_eq!(
            completed_batch.status,
            BatchStatus::Completed,
            "Mock implementation forcibly completes the batch"
        );

        info!("Trying to retrieve a non-existent file...");
        let file_content_result = mock.file_content("non_existent_file").await;
        assert!(file_content_result.is_err(), "Should fail for unknown file ID");
    }

    /// Exhaustive test suite for the `OpenAIClientHandle` struct.
    /// We'll verify that:
    /// 1. `new()` function properly checks the `OPENAI_API_KEY` environment variable.
    /// 2. The aggregator trait `LanguageModelClientInterface<E>` is satisfiable.
    /// 3. The delegated methods `batches()` and `files()` are publicly callable.
    ///
    /// NOTE on std::env manipulations: some toolchains or configs may treat
    /// `remove_var()` and `set_var()` as "unsafe". If your environment forbids
    /// them, you can remove or adjust the tests that rely on them. Below, we
    /// wrap them in an `unsafe { ... }` block to silence E0133, acknowledging
    /// that in real code you may do something else.
    #[traced_test]
    fn test_new_openai_client_handle_env_var_missing() {
        info!("Beginning test_new_openai_client_handle_env_var_missing");

        let original_api_key = std::env::var("OPENAI_API_KEY").ok();
        if original_api_key.is_some() {
            trace!("OPENAI_API_KEY is currently set; removing it for this test...");
            unsafe {
                std::env::remove_var("OPENAI_API_KEY");
            }
        }

        // If it's still present, we skip:
        if std::env::var("OPENAI_API_KEY").is_ok() {
            warn!("Skipping test_new_openai_client_handle_env_var_missing because we couldn't unset OPENAI_API_KEY in this environment.");
            return;
        }

        // Now calling `new()` should panic because there's no env var
        let result = std::panic::catch_unwind(|| {
            MockLanguageModelClient::<MockBatchClientError>::new();
        });
        debug!("Result from calling new() without the env var: {:?}", result);

        assert!(
            result.is_err(),
            "Expected new() to panic when OPENAI_API_KEY is unset"
        );

        // Restore original var if any
        if let Some(val) = original_api_key {
            trace!("Restoring OPENAI_API_KEY...");
            unsafe {
                std::env::set_var("OPENAI_API_KEY", val);
            }
        }

        info!("test_new_openai_client_handle_env_var_missing passed (or skipped).");
    }


    #[traced_test]
    fn test_new_openai_client_handle_env_var_present() {
        info!("Beginning test_new_openai_client_handle_env_var_present");

        // Save original var
        let original_api_key = std::env::var("OPENAI_API_KEY").ok();
        let test_value = "test_openai_api_key_12345";

        trace!("Temporarily setting OPENAI_API_KEY to {}", test_value);
        unsafe {
            std::env::set_var("OPENAI_API_KEY", test_value);
        }

        let result = std::panic::catch_unwind(|| {
            MockLanguageModelClient::<MockBatchClientError>::new()
        });
        debug!("Result from calling new() with env var set: {:?}", result);

        // We expect success
        assert!(
            result.is_ok(),
            "Expected new() to succeed when OPENAI_API_KEY is set"
        );
        let handle = result.unwrap();
        debug!("Created handle: {:?}", handle);

        // Cleanup: restore any original var or remove it entirely
        match original_api_key {
            Some(val) => {
                trace!("Restoring original OPENAI_API_KEY value...");
                unsafe {
                    std::env::set_var("OPENAI_API_KEY", val);
                }
            }
            None => {
                trace!("Removing OPENAI_API_KEY to restore no-value state...");
                unsafe {
                    std::env::remove_var("OPENAI_API_KEY");
                }
            }
        }

        info!("test_new_openai_client_handle_env_var_present passed.");
    }

    #[traced_test]
    fn test_delegate_methods() {
        info!("Beginning test_delegate_methods");

        // Provide a "safe" dummy so we won't panic:
        let original_api_key = std::env::var("OPENAI_API_KEY").ok();
        unsafe {
            std::env::set_var("OPENAI_API_KEY", "mock_test_key");
        }

        let handle: MockLanguageModelClient<MockBatchClientError> = std::panic::catch_unwind(|| {
            MockLanguageModelClient::<MockBatchClientError>::new()
        })
        .expect("Should not panic for mock_test_key");

        debug!("Successfully created handle: {:?}", handle);

        // Now test the delegated methods
        // (In real usage, these do network calls. We just confirm they compile & run.)
        let _batches = handle.batches();
        let _files = handle.files();

        info!("Handle's delegated methods (batches, files) are callable without error.");

        // Cleanup environment
        match original_api_key {
            Some(val) => unsafe { std::env::set_var("OPENAI_API_KEY", val) },
            None => unsafe { std::env::remove_var("OPENAI_API_KEY") },
        }

        info!("test_delegate_methods passed.");
    }

    #[traced_test]
    fn test_aggregator_trait_compatibility() {
        info!("Beginning test_aggregator_trait_compatibility");
        trace!("Ensuring that `MockLanguageModelClient` can be used as `LanguageModelClientInterface` object.");

        let original_api_key = std::env::var("OPENAI_API_KEY").ok();
        unsafe {
            std::env::set_var("OPENAI_API_KEY", "some_mock_key");
        }

        let handle_arc = Arc::new(std::panic::catch_unwind(|| {
            MockLanguageModelClient::<MockBatchClientError>::new()
        })
        .expect("Should not panic with some_mock_key"));

        let client_interface_arc: Arc<dyn LanguageModelClientInterface<MockBatchClientError>> =
            handle_arc as Arc<dyn LanguageModelClientInterface<MockBatchClientError>>;
        debug!(
            "We can coerce the handle into the aggregator trait object: {:?}",
            client_interface_arc
        );

        // Typically, you'd call aggregator methods here. We won't do so to avoid real network calls.

        // Cleanup
        match original_api_key {
            Some(val) => unsafe { std::env::set_var("OPENAI_API_KEY", val) },
            None => unsafe { std::env::remove_var("OPENAI_API_KEY") },
        }

        info!("test_aggregator_trait_compatibility passed.");
    }
}
