crate::ix!();

#[async_trait]
impl<E> RetrieveBatchById for OpenAIClientHandle<E>
where
    E: Debug + Send + Sync + From<OpenAIClientError>, // so we can do `.map_err(E::from)?`
{
    type Error = E;

    async fn retrieve_batch(&self, batch_id: &str) -> Result<Batch, Self::Error> {
        info!("retrieving batch {} from online", batch_id);

        // The underlying call returns `Result<Batch, OpenAIApiError>` 
        // or `Result<Batch, OpenAIClientError>`? Let’s assume it’s an OpenAI error:
        let batch = self.batches().retrieve(batch_id)
            .await
            .map_err(|openai_err| E::from(OpenAIClientError::OpenAIError(openai_err)))?;

        Ok(batch)
    }
}

#[cfg(test)]
mod retrieve_batch_by_id_tests {
    use super::*;
    use futures::executor::block_on;
    use tempfile::tempdir;
    use tracing::{debug, error, info, trace, warn};
    use std::sync::Arc;

    /// Exhaustive test suite for the `RetrieveBatchById` implementation on `OpenAIClientHandle`.
    /// We use the mock client to simulate various scenarios (success, error, invalid input, etc).
    #[traced_test]
    async fn test_retrieve_batch_by_id_success() {
        info!("Beginning test_retrieve_batch_by_id_success");
        trace!("Constructing mock client...");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        debug!("Mock client built: {:?}", mock_client);

        let batch_id = "valid_batch_id";

        trace!("Calling retrieve_batch on mock_client with batch_id={}", batch_id);
        let result = mock_client.retrieve_batch(batch_id).await;
        debug!("Result from retrieve_batch: {:?}", result);

        // Expect an Ok result for a valid batch
        assert!(
            result.is_ok(),
            "Expected retrieve_batch to succeed with a valid batch_id"
        );
        let batch = result.unwrap();
        pretty_assert_eq!(
            batch.id, batch_id,
            "Retrieved batch should match the requested batch_id"
        );
        info!("test_retrieve_batch_by_id_success passed.");
    }

    #[traced_test]
    async fn test_retrieve_batch_by_id_empty_input() {
        info!("Beginning test_retrieve_batch_by_id_empty_input");
        trace!("Constructing mock client...");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        debug!("Mock client built: {:?}", mock_client);

        let batch_id = "";

        trace!("Calling retrieve_batch with an empty batch_id");
        let result = mock_client.retrieve_batch(batch_id).await;
        debug!("Result from retrieve_batch: {:?}", result);

        // We now short-circuit in retrieve_batch: if batch_id.is_empty(), return error
        assert!(
            result.is_err(),
            "Expected retrieve_batch to produce an error for empty batch_id"
        );
        info!("test_retrieve_batch_by_id_empty_input passed.");
    }

    #[traced_test]
    async fn test_retrieve_batch_by_id_openai_api_error() {
        info!("Beginning test_retrieve_batch_by_id_openai_api_error");
        trace!("Constructing mock client that simulates an OpenAI error...");
        let mock_client = {
            let mut builder = MockLanguageModelClientBuilder::<MockBatchClientError>::default();
            builder.build().unwrap()
        };
        debug!("Mock client built: {:?}", mock_client);

        let batch_id = "trigger_api_error";

        trace!("Calling retrieve_batch expecting an API error scenario...");
        let result = mock_client.retrieve_batch(batch_id).await;
        debug!("Result from retrieve_batch: {:?}", result);

        // Because we “trigger_api_error,” the mock forcibly returns an OpenAIError.
        assert!(
            result.is_err(),
            "Expected retrieve_batch to return an error due to OpenAI API error"
        );
        info!("test_retrieve_batch_by_id_openai_api_error passed.");
    }

    #[traced_test]
    async fn test_retrieve_batch_by_id_other_error() {
        info!("Beginning test_retrieve_batch_by_id_other_error");
        trace!("Constructing mock client that simulates a different kind of error...");
        let mock_client = {
            let mut builder = MockLanguageModelClientBuilder::<MockBatchClientError>::default();
            builder.build().unwrap()
        };
        debug!("Mock client built: {:?}", mock_client);

        let batch_id = "trigger_other_error";

        trace!("Calling retrieve_batch expecting a non-OpenAI error scenario...");
        let result = mock_client.retrieve_batch(batch_id).await;
        debug!("Result from retrieve_batch: {:?}", result);

        // Because we “trigger_other_error,” the mock forcibly returns a std::io::Error
        assert!(
            result.is_err(),
            "Expected retrieve_batch to return an error from a non-OpenAI scenario"
        );
        info!("test_retrieve_batch_by_id_other_error passed.");
    }
}
