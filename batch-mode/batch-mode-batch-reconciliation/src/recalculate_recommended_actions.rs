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
