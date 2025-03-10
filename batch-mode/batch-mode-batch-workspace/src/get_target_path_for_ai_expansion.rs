crate::ix!();

pub trait GetTargetPathForAIExpansion {

    fn target_path_for_ai_json_expansion(
        &self, 
        target_dir:            impl AsRef<Path>,
        expected_content_type: &ExpectedContentType,

    ) -> PathBuf;
}

impl GetTargetPathForAIExpansion for CamelCaseTokenWithComment {

    fn target_path_for_ai_json_expansion(
        &self, 
        target_dir:            impl AsRef<Path>,
        expected_content_type: &ExpectedContentType,

    ) -> PathBuf {

        // Convert 'token_name' to snake_case
        let snake_token_name = to_snake_case(&self.name());

        // Determine the output filename based on custom_id
        // You can customize this as needed, e.g., using token names
        let filename = format!("{}.json", snake_token_name);

        target_dir.as_ref().to_path_buf().join(filename)
    }
}
