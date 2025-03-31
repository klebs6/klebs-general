// ---------------- [ File: batch-mode-batch-workspace-interface/src/calculate_unseen.rs ]
crate::ix!();

pub trait CalculateUnseenInputs<T> {
    fn calculate_unseen_inputs(
        &self, 
        inputs: &[T], 
        expected_content_type: &ExpectedContentType
    ) -> Vec<T>;
}

impl<W,T> CalculateUnseenInputs<T> for W 
where W: FindSimilarTargetPath + GetTargetDir,
      T: GetTargetPathForAIExpansion + Clone + Debug + Display + Named,
{
    /// Internal helper. Identifies newly seen tokens.
    fn calculate_unseen_inputs(
        &self, 
        inputs:                &[T], 
        expected_content_type: &ExpectedContentType

    ) -> Vec<T> {

        let target_dir = self.get_target_dir();

        trace!("In target_dir={}, calculating unseen inputs:", target_dir.display());

        let mut unseen: Vec<T> = Vec::new();

        for tok in inputs {

            let target_path = tok.target_path_for_ai_json_expansion(
                &target_dir,
                expected_content_type
            );

            trace!("target_path={:?} for token={}", target_path, tok);

            if !target_path.exists() {

                if let Some(similar_path) = self.find_similar_target_path(&target_path) {

                    warn!(
                        "Skipping token '{}': target path '{}' is similar to existing '{}'.",
                        tok.name(),
                        target_path.display(),
                        similar_path.display()
                    );

                    // Skip this token
                    continue;
                }

                unseen.push(tok.clone());
            }
        }

        info!("Unseen input tokens calculated: {:#?}",unseen);

        unseen
    }
}

// tests for this are in batch-mode-batch-workspace
