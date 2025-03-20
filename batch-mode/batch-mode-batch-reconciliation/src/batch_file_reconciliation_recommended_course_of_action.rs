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

#[cfg(test)]
mod batch_file_reconciliation_recommended_course_of_action_tests {
    use super::*;

    #[traced_test]
    async fn test_try_from_triple_all_none() {

        let mock_index = BatchIndex::from(123u64);

        let workspace  = BatchWorkspace::new_temp().await.expect("expected to get our workspace") as Arc<dyn BatchWorkspaceInterface>;

        let triple = BatchFileTripleBuilder::default()
            .index(mock_index.clone())
            .workspace(workspace)
            .build()
            .unwrap();

        let result = BatchFileReconciliationRecommendedCourseOfAction::try_from(&triple);
        assert!(result.is_err(), "Expected error if all files are None");
        match result {
            Err(BatchReconciliationError::BatchWorkspaceError(e)) => {
                match e {
                    BatchWorkspaceError::NoBatchFileTripleAtIndex { index } => {
                        pretty_assert_eq!(index, mock_index);
                    }
                    _ => panic!("Unexpected error variant for all_none scenario"),
                }
            }
            other => panic!("Unexpected result: {:?}", other),
        }
    }

    #[traced_test]
    async fn test_try_from_triple_missing_input_but_has_output() {

        let workspace  = BatchWorkspace::new_temp().await.expect("expected to get our workspace") as Arc<dyn BatchWorkspaceInterface>;

        let triple = BatchFileTripleBuilder::default()
            .index(BatchIndex::from(9999u64))
            .output::<PathBuf>("some_output.json".into())
            .workspace(workspace)
            .build()
            .unwrap();

        let result = BatchFileReconciliationRecommendedCourseOfAction::try_from(&triple);
        assert!(result.is_err(), "Should fail if input is missing but output exists");
        match result {
            Err(BatchReconciliationError::MissingBatchInputFileButOthersExist { index, output, error }) => {
                pretty_assert_eq!(index.as_u64(), Some(9999u64));
                pretty_assert_eq!(output, Some("some_output.json".into()));
                pretty_assert_eq!(error, None);
            }
            other => panic!("Unexpected error variant for missing input scenario: {:?}", other),
        }
    }

    #[traced_test]
    async fn test_try_from_triple_input_only() {

        let workspace  = BatchWorkspace::new_temp().await.expect("expected to get our workspace") as Arc<dyn BatchWorkspaceInterface>;

        let triple = BatchFileTripleBuilder::default()
            .index(BatchIndex::from(1000u64))
            .input::<PathBuf>("input.json".into())
            .workspace(workspace)
            .build()
            .unwrap();

        let result = BatchFileReconciliationRecommendedCourseOfAction::try_from(&triple);
        assert!(result.is_ok(), "Input-only scenario should be Ok");
        let steps = result.unwrap().steps().to_vec();
        // Expect: [CheckForBatchOutputAndErrorFileOnline, RecalculateRecommendedCourseOfActionIfTripleChanged]
        pretty_assert_eq!(
            steps,
            vec![
                BatchFileTripleReconciliationOperation::CheckForBatchOutputAndErrorFileOnline,
                BatchFileTripleReconciliationOperation::RecalculateRecommendedCourseOfActionIfTripleChanged
            ]
        );
    }

    #[traced_test]
    async fn test_try_from_triple_input_output() {

        let workspace  = BatchWorkspace::new_temp().await.expect("expected to get our workspace") as Arc<dyn BatchWorkspaceInterface>;

        let triple = BatchFileTripleBuilder::default()
            .index(BatchIndex::from(1u64))
            .input::<PathBuf>("input.json".into())
            .output::<PathBuf>("output.json".into())
            .workspace(workspace)
            .build()
            .unwrap();


        let result = BatchFileReconciliationRecommendedCourseOfAction::try_from(&triple);
        assert!(result.is_ok(), "Input+Output scenario should be Ok");
        let steps = result.unwrap().steps().to_vec();
        // Expect: [EnsureInputRequestIdsMatchOutputRequestIds, ProcessBatchOutputFile, MoveBatchInputAndOutputToTheDoneDirectory]
        pretty_assert_eq!(
            steps,
            vec![
                BatchFileTripleReconciliationOperation::EnsureInputRequestIdsMatchOutputRequestIds,
                BatchFileTripleReconciliationOperation::ProcessBatchOutputFile,
                BatchFileTripleReconciliationOperation::MoveBatchInputAndOutputToTheDoneDirectory
            ]
        );
    }

    #[traced_test]
    async fn test_try_from_triple_input_error() {

        let workspace  = BatchWorkspace::new_temp().await.expect("expected to get our workspace") as Arc<dyn BatchWorkspaceInterface>;

        let triple = BatchFileTripleBuilder::default()
            .index(BatchIndex::from(2u64))
            .input::<PathBuf>("input.json".into())
            .error::<PathBuf>("error.json".into())
            .workspace(workspace)
            .build()
            .unwrap();

        let result = BatchFileReconciliationRecommendedCourseOfAction::try_from(&triple);
        assert!(result.is_ok(), "Input+Error scenario should be Ok");
        let steps = result.unwrap().steps().to_vec();
        // Expect: [EnsureInputRequestIdsMatchErrorRequestIds, ProcessBatchErrorFile, MoveBatchInputAndErrorToTheDoneDirectory]
        pretty_assert_eq!(
            steps,
            vec![
                BatchFileTripleReconciliationOperation::EnsureInputRequestIdsMatchErrorRequestIds,
                BatchFileTripleReconciliationOperation::ProcessBatchErrorFile,
                BatchFileTripleReconciliationOperation::MoveBatchInputAndErrorToTheDoneDirectory
            ]
        );
    }

    #[traced_test]
    async fn test_try_from_triple_input_output_error() {

        let workspace  = BatchWorkspace::new_temp().await.expect("expected to get our workspace") as Arc<dyn BatchWorkspaceInterface>;

        let triple = BatchFileTripleBuilder::default()
            .index(BatchIndex::from(3u64))
            .input::<PathBuf>("input.json".into())
            .output::<PathBuf>("output.json".into())
            .error::<PathBuf>("error.json".into())
            .workspace(workspace)
            .build()
            .unwrap();

        let result = BatchFileReconciliationRecommendedCourseOfAction::try_from(&triple);
        assert!(result.is_ok(), "Input+Output+Error scenario should be Ok");
        let steps = result.unwrap().steps().to_vec();
        // Expect: [EnsureInputRequestIdsMatchOutputRequestIdsCombinedWithErrorRequestIds, ProcessBatchErrorFile, ProcessBatchOutputFile, MoveBatchTripleToTheDoneDirectory]
        pretty_assert_eq!(
            steps,
            vec![
                BatchFileTripleReconciliationOperation::EnsureInputRequestIdsMatchOutputRequestIdsCombinedWithErrorRequestIds,
                BatchFileTripleReconciliationOperation::ProcessBatchErrorFile,
                BatchFileTripleReconciliationOperation::ProcessBatchOutputFile,
                BatchFileTripleReconciliationOperation::MoveBatchTripleToTheDoneDirectory
            ]
        );
    }
}
