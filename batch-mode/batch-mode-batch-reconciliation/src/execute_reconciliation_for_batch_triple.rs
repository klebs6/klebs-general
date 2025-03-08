// ---------------- [ File: src/execute_reconciliation_for_batch_triple.rs ]
crate::ix!();

impl<E> ExecuteReconciliationOperation<E>
for BatchFileTriple where BatchDownloadError: From<E>,
{
    async fn execute_reconciliation_operation(
        &mut self,
        client:                 &dyn LanguageModelClientInterface<E>,
        operation:              &BatchFileTripleReconciliationOperation,
        expected_content_type:  &ExpectedContentType,
        process_output_file_fn: &OutputFileFn,
        process_error_file_fn:  &ErrorFileFn,
    ) -> Result<Option<BatchFileReconciliationRecommendedCourseOfAction>, BatchReconciliationError>
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
                });
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
