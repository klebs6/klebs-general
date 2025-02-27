// ---------------- [ File: src/batch_file_reconciliation_recommended_course_of_action.rs ]
crate::ix!();

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct BatchFileReconciliationRecommendedCourseOfAction {
    steps: Vec<BatchFileTripleReconciliationOperation>,
}

impl BatchFileReconciliationRecommendedCourseOfAction {
    pub fn steps(&self) -> &[BatchFileTripleReconciliationOperation] {
        &self.steps
    }
}

impl From<Vec<BatchFileTripleReconciliationOperation>> for BatchFileReconciliationRecommendedCourseOfAction {

    fn from(steps: Vec<BatchFileTripleReconciliationOperation>) -> Self {
        Self { steps }
    }
}

impl TryFrom<&BatchFileTriple> for BatchFileReconciliationRecommendedCourseOfAction {

    type Error = BatchReconciliationError;

    // here we are assuming the results of each step are a success and that something happened
    //
    // in case we get an error, for now the course of action will be to log the error and skip
    // processing this batch triple.
    //
    fn try_from(triple: &BatchFileTriple) -> Result<BatchFileReconciliationRecommendedCourseOfAction, BatchReconciliationError> {
        if triple.all_are_none() {
            return Err(BatchWorkspaceError::NoBatchFileTripleAtIndex {
                index: triple.index().clone(),
            }.into());
        }

        if triple.input().is_none() {
            return Err(BatchReconciliationError::MissingBatchInputFileButOthersExist {
                index: triple.index().clone(),
                output: triple.output().clone(),
                error: triple.error().clone(),
            });
        }

        use BatchFileTripleReconciliationOperation::*;
        let steps = match BatchFileState::from(triple) {
            BatchFileState::InputOutputError => {
                warn!("Both output and error files are present for batch {:?}", triple.index());
                vec![
                    EnsureInputRequestIdsMatchOutputRequestIdsCombinedWithErrorRequestIds,
                    ProcessBatchErrorFile,
                    ProcessBatchOutputFile,
                    MoveBatchTripleToTheDoneDirectory,
                ]
            }
            BatchFileState::InputOutput => {
                vec![
                    EnsureInputRequestIdsMatchOutputRequestIds,
                    ProcessBatchOutputFile,
                    MoveBatchInputAndOutputToTheDoneDirectory,
                ]
            }
            BatchFileState::InputError => {
                warn!("Error file present but no output file for batch {:?}", triple.index());
                vec![
                    EnsureInputRequestIdsMatchErrorRequestIds,
                    ProcessBatchErrorFile,
                    MoveBatchInputAndErrorToTheDoneDirectory,
                ]

            }
            BatchFileState::InputOnly => {
                warn!("Neither output nor error files are present for batch {:?}", triple.index());
                vec![
                    CheckForBatchOutputAndErrorFileOnline,
                    RecalculateRecommendedCourseOfActionIfTripleChanged,
                ]
            }
        };

        Ok(BatchFileReconciliationRecommendedCourseOfAction::from(steps))
    }
}
