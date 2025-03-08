// ---------------- [ File: src/execute_reconciliation.rs ]
crate::ix!();

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatchErrorFileProcessingOperation {
    LogErrors,
    RetryFailedRequests,
    // Add other operations as needed
}

pub trait ExecuteReconciliationOperation<OutputF,ErrorF,OFut,EFut,E,C>
where
    OutputF: Fn(&BatchFileTriple, &dyn BatchWorkspaceInterface, &ExpectedContentType) -> OFut + Send + Sync,
    ErrorF:  Fn(&BatchFileTriple, &[BatchErrorFileProcessingOperation]) -> EFut + Send + Sync,
    OFut:    Future<Output = Result<(), BatchOutputProcessingError>> + Send,
    EFut:    Future<Output = Result<(), BatchErrorProcessingError>> + Send,
    C:       LanguageModelClientInterface<E>
{
    async fn execute_reconciliation_operation(
        &mut self,
        client:                 &C,
        operation:              &BatchFileTripleReconciliationOperation,
        expected_content_type:  &ExpectedContentType,
        process_output_file_fn: &OutputF,
        process_error_file_fn:  &ErrorF,
    ) -> Result<Option<BatchFileReconciliationRecommendedCourseOfAction>, BatchReconciliationError>;
}
