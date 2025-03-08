// ---------------- [ File: src/execute_reconciliation_for_batch_triple.rs ]
crate::ix!();

impl<OutputF,ErrorF,OFut,EFut,E,C> ExecuteReconciliationOperation<OutputF,ErrorF,OFut,EFut,E,C>
for BatchFileTriple 
where
    OutputF:            Fn(&BatchFileTriple, &dyn BatchWorkspaceInterface, &ExpectedContentType) -> OFut + Send + Sync,
    ErrorF:             Fn(&BatchFileTriple, &[BatchErrorFileProcessingOperation]) -> EFut + Send + Sync,
    OFut:               Future<Output = Result<(), BatchOutputProcessingError>> + Send,
    EFut:               Future<Output = Result<(), BatchErrorProcessingError>> + Send,
    C:                  LanguageModelClientInterface<E>,
    BatchDownloadError: From<E>
{
    async fn execute_reconciliation_operation(
        &mut self,
        client:                 &C,
        operation:              &BatchFileTripleReconciliationOperation,
        expected_content_type:  &ExpectedContentType,
        process_output_file_fn: &OutputF,
        process_error_file_fn:  &ErrorF,
    ) -> Result<Option<BatchFileReconciliationRecommendedCourseOfAction>, BatchReconciliationError>
    {
        let workspace = self.workspace();

        info!(
            "executing reconciliation operation {:?} for batch {:#?}",
            operation, self
        );

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
                process_error_file_fn(self,&operations).await?;
            }
            ProcessBatchOutputFile => {
                process_output_file_fn(self,&**workspace, expected_content_type).await?;
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

        Ok(new_recommended_actions)
    }
}
