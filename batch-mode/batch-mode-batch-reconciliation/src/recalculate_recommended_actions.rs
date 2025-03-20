// ---------------- [ File: src/recalculate_recommended_actions.rs ]
crate::ix!();

pub trait RecalculateRecommendedActions {
    fn recalculate_recommended_actions(&self) 
        -> Result<BatchFileReconciliationRecommendedCourseOfAction, BatchReconciliationError>;
}

impl RecalculateRecommendedActions for BatchFileTriple {

    // Implement recalculate_recommended_actions method
    fn recalculate_recommended_actions(&self) -> Result<BatchFileReconciliationRecommendedCourseOfAction, BatchReconciliationError> {
        // Re-run the logic to determine the next steps based on the updated triple
        BatchFileReconciliationRecommendedCourseOfAction::try_from(self)
    }
}

#[cfg(test)]
mod recalculate_recommended_actions_tests {
    use super::*;

    #[traced_test]
    async fn test_recalculate_recommended_actions() {

        let workspace: Arc<dyn BatchWorkspaceInterface> = BatchWorkspace::new_temp().await.expect("expected workspace");

        let triple = BatchFileTripleBuilder::default()
            .index(BatchIndex::from(999u64))
            .input::<PathBuf>("input.json".into())
            .workspace(workspace)
            .build()
            .unwrap();

        let recommended = triple.recalculate_recommended_actions();
        assert!(recommended.is_ok());
        let steps = recommended.unwrap().steps().to_vec();
        pretty_assert_eq!(
            steps,
            vec![
                BatchFileTripleReconciliationOperation::CheckForBatchOutputAndErrorFileOnline,
                BatchFileTripleReconciliationOperation::RecalculateRecommendedCourseOfActionIfTripleChanged,
            ]
        );
    }

    #[traced_test]
    fn test_recalculate_recommended_actions_failure() {
        // If the triple is all None => recalc should fail
        let triple = BatchFileTriple::new_for_test_empty();
        let recommended = triple.recalculate_recommended_actions();
        assert!(recommended.is_err());
        match recommended.err().unwrap() {
            BatchReconciliationError::BatchWorkspaceError(BatchWorkspaceError::NoBatchFileTripleAtIndex{index}) => {
                // This is correct
                pretty_assert_eq!(index.as_u64(), triple.index().as_u64());
            },
            other => panic!("Unexpected error variant: {:?}", other),
        }
    }
}
