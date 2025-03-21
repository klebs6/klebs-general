// ---------------- [ File: src/execute_reconciliation_for_batch_triple.rs ]
crate::ix!();

#[async_trait]
impl<E> ExecuteReconciliationOperation<E>
for BatchFileTriple 
where E
: From<BatchReconciliationError> 
+ From<BatchDownloadError> 
+ From<OpenAIClientError> 
+ From<BatchMetadataError> 
+ From<std::io::Error>
+ From<FileMoveError>
+ From<BatchOutputProcessingError>
+ From<BatchErrorProcessingError>
+ From<BatchValidationError>
+ Display
+ Debug
{
    async fn execute_reconciliation_operation(
        &mut self,
        client:                 &dyn LanguageModelClientInterface<E>,
        operation:              &BatchFileTripleReconciliationOperation,
        expected_content_type:  &ExpectedContentType,
        process_output_file_fn: &BatchWorkflowProcessOutputFileFn,
        process_error_file_fn:  &BatchWorkflowProcessErrorFileFn,
    ) -> Result<Option<BatchFileReconciliationRecommendedCourseOfAction>, E>
    {
        info!(
            "Preparing to execute reconciliation operation {:?} for batch {:?}",
            operation,
            self.index()
        );

        let workspace = self.workspace();
        let mut new_recommended_actions = None;

        use BatchFileTripleReconciliationOperation::*;

        match operation {
            EnsureInputRequestIdsMatchErrorRequestIds => {
                self.ensure_input_matches_error().await?;
            }
            EnsureInputRequestIdsMatchOutputRequestIds => {
                self.ensure_input_matches_output().await?;
            }
            EnsureInputRequestIdsMatchOutputRequestIdsCombinedWithErrorRequestIds => {
                self.ensure_input_matches_output_and_error().await?;
            }
            ProcessBatchErrorFile => {
                let operations = vec![
                    BatchErrorFileProcessingOperation::LogErrors,
                    BatchErrorFileProcessingOperation::RetryFailedRequests,
                ];
                process_error_file_fn(self, &operations).await?;
            }
            ProcessBatchOutputFile => {
                process_output_file_fn(self, &**workspace, expected_content_type).await?;
            }
            MoveBatchInputAndErrorToTheDoneDirectory => {
                self.move_input_and_error_to_done().await?;
            }
            MoveBatchInputAndOutputToTheDoneDirectory => {
                self.move_input_and_output_to_done().await?;
            }
            MoveBatchTripleToTheDoneDirectory => {
                self.move_all_to_done().await?;
            }
            CheckForBatchOutputAndErrorFileOnline => {
                self.check_for_and_download_output_and_error_online(client).await?;
                new_recommended_actions = Some(self.recalculate_recommended_actions()?);
            }
            RecalculateRecommendedCourseOfActionIfTripleChanged => {
                new_recommended_actions = Some(self.recalculate_recommended_actions()?);
            }
            _ => {
                return Err(BatchReconciliationError::OperationNotImplemented {
                    operation: *operation,
                }.into());
            }
        }

        info!(
            "Reconciliation operation {:?} for batch {:?} completed successfully",
            operation,
            self.index()
        );

        Ok(new_recommended_actions)
    }
}


#[cfg(test)]
mod execute_reconciliation_for_batch_triple_tests {
    use super::*;
    use std::{
        future::Future,
        pin::Pin,
        fs,
    };

    /// Must match EXACTLY the type alias in `execute_reconciliation.rs`
    fn mock_process_output<'a>(
        _triple: &'a BatchFileTriple,
        _workspace: &'a (dyn BatchWorkspaceInterface + 'a),
        _content_type: &'a ExpectedContentType,
    ) -> Pin<Box<dyn Future<Output = Result<(), BatchOutputProcessingError>> + Send + 'a>> {
        Box::pin(async move {
            debug!("mock_process_output called");
            Ok(())
        })
    }

    /// Must match EXACTLY the type alias in `execute_reconciliation.rs`
    fn mock_process_error<'a>(
        _triple: &'a BatchFileTriple,
        _operations: &'a [BatchErrorFileProcessingOperation],
    ) -> Pin<Box<dyn Future<Output = Result<(), BatchErrorProcessingError>> + Send + 'a>> {
        Box::pin(async move {
            debug!("mock_process_error called");
            Ok(())
        })
    }

    const MOCK_PROCESS_OUTPUT: BatchWorkflowProcessOutputFileFn = mock_process_output;
    const MOCK_PROCESS_ERROR:  BatchWorkflowProcessErrorFileFn  = mock_process_error;

    #[traced_test]
    fn test_execute_reconciliation_operation_move_batch_input_and_output_to_done() {
        // Use a real tokio runtime so that tokio::fs calls won't panic.
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut triple = BatchFileTriple::new_for_test_empty();
            triple.set_input_path(Some("my_input.json".into()));
            triple.set_output_path(Some("my_output.json".into()));

            // Create the input/output files so they actually exist
            fs::write("my_input.json", b"fake input").unwrap();
            fs::write("my_output.json", b"fake output").unwrap();

            let client_mock = Arc::new(
                MockLanguageModelClientBuilder::<MockBatchClientError>::default()
                    .build()
                    .unwrap(),
            ) as Arc<dyn LanguageModelClientInterface<MockBatchClientError>>;

            let operation = BatchFileTripleReconciliationOperation::MoveBatchInputAndOutputToTheDoneDirectory;
            let ect = ExpectedContentType::JsonLines;

            let result = triple.execute_reconciliation_operation(
                client_mock.as_ref(),
                &operation,
                &ect,
                &MOCK_PROCESS_OUTPUT,
                &MOCK_PROCESS_ERROR,
            ).await;

            assert!(result.is_ok(), "Should succeed moving input+output to done directory");
            assert_eq!(
                result.unwrap(),
                None,
                "Move ops typically return no new actions"
            );

            // Cleanup if desired
        });
    }

    #[traced_test]
    fn test_execute_reconciliation_operation_process_batch_output_file() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut triple = BatchFileTriple::new_for_test_empty();
            triple.set_input_path(Some("my_input.json".into()));
            triple.set_output_path(Some("my_output.json".into()));

            // Create them so code that tries to read them won't fail
            fs::write("my_input.json", b"fake input").unwrap();
            fs::write("my_output.json", b"fake output").unwrap();

            let client_mock = Arc::new(
                MockLanguageModelClientBuilder::<MockBatchClientError>::default()
                    .build()
                    .unwrap(),
            ) as Arc<dyn LanguageModelClientInterface<MockBatchClientError>>;

            let operation = BatchFileTripleReconciliationOperation::ProcessBatchOutputFile;
            let ect = ExpectedContentType::JsonLines;

            let result = triple.execute_reconciliation_operation(
                client_mock.as_ref(),
                &operation,
                &ect,
                &MOCK_PROCESS_OUTPUT,
                &MOCK_PROCESS_ERROR,
            ).await;

            assert!(result.is_ok(), "Should succeed processing batch output file");
            assert_eq!(
                result.unwrap(),
                None,
                "No follow-up actions from the default mock processing"
            );
        });
    }

    #[traced_test]
    fn test_execute_reconciliation_operation_check_for_batch_output_and_error_file_online() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut triple = BatchFileTriple::new_for_test_empty();
            triple.set_input_path(Some("my_input.json".into()));

            // Provide *both* "batch_id" and "input_file_id" so it doesn't fail
            // with missing field errors:
            fs::write(
                "mock_metadata_9999.json",
                r#"{"batch_id":"some_mock_batch_id","input_file_id":"fake_input_file_id_9999"}"#
            ).unwrap();

            // Also create the input file so it doesn't fail if it tries to read it
            fs::write("my_input.json", b"fake input").unwrap();

            let client_mock = Arc::new(
                MockLanguageModelClientBuilder::<MockBatchClientError>::default()
                    .build()
                    .unwrap(),
            ) as Arc<dyn LanguageModelClientInterface<MockBatchClientError>>;

            let operation = BatchFileTripleReconciliationOperation::CheckForBatchOutputAndErrorFileOnline;
            let ect = ExpectedContentType::JsonLines;

            let result = triple.execute_reconciliation_operation(
                client_mock.as_ref(),
                &operation,
                &ect,
                &MOCK_PROCESS_OUTPUT,
                &MOCK_PROCESS_ERROR,
            ).await;

            assert!(result.is_ok(), "Should succeed checking for files online");
            let maybe_new_actions = result.unwrap();
            debug!(
                "CheckForBatchOutputAndErrorFileOnline => returned new actions: {:?}",
                maybe_new_actions
            );
        });
    }
}
