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

    // A mock "process output" function
    async fn mock_process_output(
        _triple: &BatchFileTriple,
        _workspace: &(dyn BatchWorkspaceInterface + '_),
        _content_type: &ExpectedContentType,
    ) -> Result<(), BatchOutputProcessingError> {
        // pretend we processed the output file
        Ok(())
    }

    // A mock "process error" function
    async fn mock_process_error(
        _triple: &BatchFileTriple,
        _operations: &[BatchErrorFileProcessingOperation],
    ) -> Result<(), BatchErrorProcessingError> {
        // pretend we processed the error file
        Ok(())
    }

    fn mock_process_output_fn<'a>(
        triple: &'a BatchFileTriple,
        workspace: &'a (dyn BatchWorkspaceInterface + 'a),
        ect: &'a ExpectedContentType,
    ) -> Pin<Box<dyn Future<Output = Result<(), BatchOutputProcessingError>> + Send + 'a>> {
        Box::pin(mock_process_output(triple, workspace, ect))
    }

    fn mock_process_error_fn<'a>(
        triple: &'a BatchFileTriple,
        ops: &'a [BatchErrorFileProcessingOperation],
    ) -> Pin<Box<dyn Future<Output = Result<(), BatchErrorProcessingError>> + Send + 'a>> {
        Box::pin(mock_process_error(triple, ops))
    }

    #[traced_test]
    fn test_execute_reconciliation_operation_move_batch_input_and_output_to_done() {
        let mut triple = BatchFileTriple::new_for_test_empty(); // or supply a real workspace
        triple.set_input_path(Some("my_input.json".into()));
        triple.set_output_path(Some("my_output.json".into()));

        let client_mock = Arc::new(MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap(),
        ) as Arc<dyn LanguageModelClientInterface<MockBatchClientError>>;

        // We test the "MoveBatchInputAndOutputToTheDoneDirectory"
        let operation = BatchFileTripleReconciliationOperation::MoveBatchInputAndOutputToTheDoneDirectory;
        let ect = ExpectedContentType::JsonLines;

        let result = block_on(triple.execute_reconciliation_operation(
            client_mock.as_ref(),
            &operation,
            &ect,
            &mock_process_output_fn,
            &mock_process_error_fn,
        ));
        assert!(result.is_ok(), "Should succeed moving input+output to done directory");
        pretty_assert_eq!(result.unwrap(), None, "No new actions are returned by default move ops");
    }

    #[traced_test]
    fn test_execute_reconciliation_operation_process_batch_output_file() {
        let mut triple = BatchFileTriple::new_for_test_empty();
        triple.set_input_path(Some("my_input.json".into()));
        triple.set_output_path(Some("my_output.json".into()));

        let client_mock = Arc::new(MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap(),
        ) as Arc<dyn LanguageModelClientInterface<MockBatchClientError>>;

        let operation = BatchFileTripleReconciliationOperation::ProcessBatchOutputFile;
        let ect = ExpectedContentType::JsonLines;

        let result = block_on(triple.execute_reconciliation_operation(
            client_mock.as_ref(),
            &operation,
            &ect,
            &mock_process_output_fn,
            &mock_process_error_fn,
        ));
        assert!(result.is_ok(), "Should succeed processing batch output file");
        pretty_assert_eq!(result.unwrap(), None, "No follow-up actions are returned by default processing");
    }

    #[traced_test]
    fn test_execute_reconciliation_operation_check_for_batch_output_and_error_file_online() {
        let mut triple = BatchFileTriple::new_for_test_empty();
        triple.set_input_path(Some("my_input.json".into()));

        // We configure the mock so that "check_for_and_download_output_and_error_online" 
        // might produce new recommended actions. For simplicity, we won't do a deep mock here:
        let client_mock = Arc::new(MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap(),
        ) as Arc<dyn LanguageModelClientInterface<MockBatchClientError>>;

        let operation = BatchFileTripleReconciliationOperation::CheckForBatchOutputAndErrorFileOnline;
        let ect = ExpectedContentType::JsonLines;

        let result = block_on(triple.execute_reconciliation_operation(
            client_mock.as_ref(),
            &operation,
            &ect,
            &mock_process_output_fn,
            &mock_process_error_fn,
        ));
        assert!(result.is_ok(), "Should succeed checking for files online");
        let maybe_new_actions = result.unwrap();
        // Because the code might recalc recommended actions if it discovered new files:
        // We'll just ensure it doesn't panic:
        debug!("maybe_new_actions => {:?}", maybe_new_actions);
    }
}

