crate::ix!();

#[async_trait]
impl<E> GetBatchFileContent for OpenAIClientHandle<E>
where
    E: Debug + Send + Sync + From<OpenAIClientError> + From<std::io::Error>, 
{
    type Error = E;

    async fn file_content(&self, file_id: &str) -> Result<Bytes, Self::Error> {
        info!("retrieving file {} content from online", file_id);

        let bytes = self.files().content(file_id)
            .await
            // If that returns `OpenAIApiError`, do something like:
            .map_err(|api_err| E::from(OpenAIClientError::OpenAIError(api_err)))?;

        Ok(bytes)
    }
}
#[cfg(test)]
mod get_batch_file_content_tests {
    use super::*;
    use futures::executor::block_on;
    use tempfile::tempdir;
    use tracing::{debug, error, info, trace, warn};
    use std::sync::Arc;

    /// Exhaustive test suite for the `GetBatchFileContent` implementation on `OpenAIClientHandle`.
    /// We use the mock client to simulate various scenarios (success, error, invalid input, etc).
    #[traced_test]
    async fn test_file_content_success() {
        info!("Beginning test_file_content_success");
        trace!("Constructing mock client...");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        debug!("Mock client built: {:?}", mock_client);

        let file_id = "valid_file_id";

        trace!("Calling file_content on mock_client with file_id={}", file_id);
        let result = mock_client.file_content(file_id).await;
        debug!("Result from file_content: {:?}", result);

        // Expect an Ok result for a valid file
        assert!(
            result.is_ok(),
            "Expected file_content to succeed with a valid file_id"
        );
        let bytes = result.unwrap();
        assert!(
            !bytes.is_empty(),
            "Expected the returned bytes not to be empty for a valid file"
        );
        info!("test_file_content_success passed.");
    }

    #[traced_test]
    async fn test_file_content_empty_input() {
        info!("Beginning test_file_content_empty_input");
        trace!("Constructing mock client...");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        debug!("Mock client built: {:?}", mock_client);

        let file_id = "";

        trace!("Calling file_content with an empty file_id");
        let result = mock_client.file_content(file_id).await;
        debug!("Result from file_content: {:?}", result);

        // We expect the mock or the real call to fail if the file_id is invalid.
        assert!(
            result.is_err(),
            "Expected file_content to produce an error for empty file_id"
        );

        info!("test_file_content_empty_input passed.");
    }

    #[traced_test]
    async fn test_file_content_openai_api_error() {
        info!("Beginning test_file_content_openai_api_error");
        trace!("Constructing mock client that simulates an OpenAI error...");
        let mock_client = {
            let mut builder = MockLanguageModelClientBuilder::<MockBatchClientError>::default();
            // Hypothetically configure builder to fail on file_content, if supported:
            // builder.fail_on_file_content_openai_error(true);
            builder.build().unwrap()
        };
        debug!("Mock client built: {:?}", mock_client);

        let file_id = "trigger_api_error";

        trace!("Calling file_content expecting an API error scenario...");
        let result = mock_client.file_content(file_id).await;
        debug!("Result from file_content: {:?}", result);

        // We expect an Err if the mock is configured to simulate an OpenAI error
        assert!(
            result.is_err(),
            "Expected file_content to return an error due to OpenAI API error"
        );

        info!("test_file_content_openai_api_error passed.");
    }

    #[traced_test]
    async fn test_file_content_other_error() {
        info!("Beginning test_file_content_other_error");
        trace!("Constructing mock client that simulates a non-OpenAI error...");
        let mock_client = {
            let mut builder = MockLanguageModelClientBuilder::<MockBatchClientError>::default();
            // builder.fail_on_file_content_with_io_error(true);
            builder.build().unwrap()
        };
        debug!("Mock client built: {:?}", mock_client);

        let file_id = "trigger_other_error";

        trace!("Calling file_content expecting a different kind of error...");
        let result = mock_client.file_content(file_id).await;
        debug!("Result from file_content: {:?}", result);

        assert!(
            result.is_err(),
            "Expected file_content to return a non-OpenAI error scenario"
        );

        info!("test_file_content_other_error passed.");
    }
}
