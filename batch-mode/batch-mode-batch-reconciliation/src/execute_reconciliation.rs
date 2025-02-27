// ---------------- [ File: src/execute_reconciliation.rs ]
crate::ix!();

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatchErrorFileProcessingOperation {
    LogErrors,
    RetryFailedRequests,
    // Add other operations as needed
}

pub async fn execute_reconciliation_operation<OutputF, ErrorF, OFut, EFut>(
    triple:                 &mut BatchFileTriple,
    client:                 &OpenAIClientHandle,
    operation:              &BatchFileTripleReconciliationOperation,
    expected_content_type:  &ExpectedContentType,
    process_output_file_fn: &OutputF,
    process_error_file_fn:  &ErrorF,

) -> Result<Option<BatchFileReconciliationRecommendedCourseOfAction>, BatchReconciliationError>
where
    OutputF: Fn(&BatchFileTriple, &dyn BatchWorkspaceInterface, &ExpectedContentType) -> OFut + Send + Sync,
    ErrorF:  Fn(&BatchFileTriple, &[BatchErrorFileProcessingOperation]) -> EFut + Send + Sync,
    OFut:    Future<Output = Result<(), BatchOutputProcessingError>> + Send,
    EFut:    Future<Output = Result<(), BatchErrorProcessingError>> + Send,
{
    let workspace = triple.workspace();

    info!(
        "executing reconciliation operation {:?} for batch {:#?}",
        operation, triple
    );

    let mut new_recommended_actions = None;

    use BatchFileTripleReconciliationOperation::*;

    match operation {
        EnsureInputRequestIdsMatchErrorRequestIds => {
            triple.ensure_input_matches_error().await?;
        }
        EnsureInputRequestIdsMatchOutputRequestIds => {
            triple.ensure_input_matches_output().await?;
        }
        EnsureInputRequestIdsMatchOutputRequestIdsCombinedWithErrorRequestIds => {
            triple.ensure_input_matches_output_and_error().await?;
        }
        ProcessBatchErrorFile => {
            let operations = vec![
                BatchErrorFileProcessingOperation::LogErrors,
                BatchErrorFileProcessingOperation::RetryFailedRequests,
            ];
            process_error_file_fn(triple, &operations).await?;
        }
        ProcessBatchOutputFile => {
            process_output_file_fn(triple, &**workspace, expected_content_type).await?;
        }
        MoveBatchInputAndErrorToTheDoneDirectory => {
            triple.move_input_and_error_to_done().await?;
        }
        MoveBatchInputAndOutputToTheDoneDirectory => {
            triple.move_input_and_output_to_done().await?;
        }
        MoveBatchTripleToTheDoneDirectory => {
            triple.move_all_to_done().await?;
        }
        CheckForBatchOutputAndErrorFileOnline => {
            check_for_and_download_output_and_error_online(triple, client).await?;
            new_recommended_actions = Some(recalculate_recommended_actions(triple)?);
        }
        RecalculateRecommendedCourseOfActionIfTripleChanged => {
            new_recommended_actions = Some(recalculate_recommended_actions(triple)?);
        }
        _ => {
            return Err(BatchReconciliationError::OperationNotImplemented {
                operation: *operation,
            });
        }
    }

    Ok(new_recommended_actions)
}
