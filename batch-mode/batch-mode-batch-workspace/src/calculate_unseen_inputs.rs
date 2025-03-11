// ---------------- [ File: src/calculate_unseen_inputs.rs ]
crate::ix!();

pub trait CalculateUnseenInputs<T> {
    fn calculate_unseen_inputs(
        &self, 
        inputs: &[T], 
        expected_content_type: &ExpectedContentType
    ) -> Vec<T>;
}

impl<T: GetTargetPathForAIExpansion + Clone + Display + Named> CalculateUnseenInputs<T> for BatchWorkspace {

    /// Internal helper. Identifies newly seen tokens.
    fn calculate_unseen_inputs(
        &self, 
        inputs:                &[T], 
        expected_content_type: &ExpectedContentType

    ) -> Vec<T> {

        let target_dir = self.target_dir();

        let mut unseen: Vec<T> = Vec::new();

        for tok in inputs {

            let target_path = tok.target_path_for_ai_json_expansion(
                &target_dir,
                expected_content_type
            );

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

        info!("Unseen input tokens calculated:");

        for token in &unseen {
            info!("{}", token);
        }

        unseen
    }
}
