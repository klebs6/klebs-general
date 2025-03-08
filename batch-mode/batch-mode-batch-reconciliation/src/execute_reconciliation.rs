// ---------------- [ File: src/execute_reconciliation.rs ]
crate::ix!();

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatchErrorFileProcessingOperation {
    LogErrors,
    RetryFailedRequests,
    // Add other operations as needed
}

pub trait ExecuteReconciliationOperation<E> where BatchDownloadError: From<E> {
    async fn execute_reconciliation_operation(
        &mut self,
        client:                 &dyn LanguageModelClientInterface<E>,
        operation:              &BatchFileTripleReconciliationOperation,
        expected_content_type:  &ExpectedContentType,
        process_output_file_fn: &OutputFileFn,
        process_error_file_fn:  &ErrorFileFn,
    ) -> Result<Option<BatchFileReconciliationRecommendedCourseOfAction>, BatchReconciliationError>;
}
