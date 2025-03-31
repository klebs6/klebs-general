// ---------------- [ File: batch-mode-batch-reconciliation/src/execute_reconciliation.rs ]
crate::ix!();

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatchErrorFileProcessingOperation {
    LogErrors,
    RetryFailedRequests,
    // Add other operations as needed
}

#[async_trait]
pub trait ExecuteReconciliationOperation<E> 
where E: From<BatchReconciliationError> + From<BatchDownloadError> 
{
    async fn execute_reconciliation_operation(
        &mut self,
        client:                 &dyn LanguageModelClientInterface<E>,
        operation:              &BatchFileTripleReconciliationOperation,
        expected_content_type:  &ExpectedContentType,
        process_output_file_fn: &BatchWorkflowProcessOutputFileFn,
        process_error_file_fn:  &BatchWorkflowProcessErrorFileFn,
    ) -> Result<Option<BatchFileReconciliationRecommendedCourseOfAction>, E>;
}
