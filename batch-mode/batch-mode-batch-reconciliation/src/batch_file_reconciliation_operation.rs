// ---------------- [ File: batch-mode-batch-reconciliation/src/batch_file_reconciliation_operation.rs ]
crate::ix!();

#[derive(Debug,Copy,Clone,PartialEq,Eq,Hash)]
pub enum BatchFileTripleReconciliationOperation {
    EnsureInputRequestIdsMatchErrorRequestIds,
    CheckForBatchErrorFileOnline,
    CheckForBatchOutputAndErrorFileOnline,
    DownloadBatchOutputAndMaybeErrorFileOnline,
    DownloadBatchOutputFileOnline,
    EnsureInputRequestIdsMatchOutputRequestIds,
    EnsureInputRequestIdsMatchOutputRequestIdsCombinedWithErrorRequestIds,
    MoveBatchInputAndErrorToTheDoneDirectory,
    MoveBatchInputAndOutputToTheDoneDirectory,
    MoveBatchTripleToTheDoneDirectory,
    ProcessBatchErrorFile,
    ProcessBatchOutputFile,
    RecalculateRecommendedCourseOfActionIfTripleChanged,
}
