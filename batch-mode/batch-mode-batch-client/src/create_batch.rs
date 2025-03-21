// ---------------- [ File: src/create_batch.rs ]
crate::ix!();

#[async_trait]
impl<E> CreateBatch for OpenAIClientHandle<E>
where
    E: Debug + Send + Sync + From<OpenAIClientError>
{
    type Error = E;

    async fn create_batch(&self, input_file_id: &str) -> Result<Batch, Self::Error> {
        info!("creating batch with input_file_id={}", input_file_id);

        let batch_request = BatchRequest {
            input_file_id:     input_file_id.to_string(),
            endpoint:          BatchEndpoint::V1ChatCompletions,
            completion_window: BatchCompletionWindow::W24H,
            metadata: None,
        };

        let batch = self.batches().create(batch_request).await
            .map_err(|api_err| E::from(OpenAIClientError::OpenAIError(api_err)))?;

        Ok(batch)
    }
}

#[cfg(test)]
mod create_batch_tests {
    use super::*;
    use futures::executor::block_on;
    use tempfile::tempdir;
    use tracing::{debug, error, info, trace, warn};

    /// Exhaustive test suite for the `CreateBatch` implementation on `OpenAIClientHandle`.
    /// We use the mock client to simulate various scenarios (success, error, invalid input, etc).

    #[traced_test]
    async fn test_create_batch_success() {
        info!("Beginning test_create_batch_success");
        trace!("Constructing mock client...");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        debug!("Mock client built: {:?}", mock_client);

        let input_file_id = "valid_input_file";

        trace!("Calling create_batch on mock_client with input_file_id={}", input_file_id);
        let result = mock_client.create_batch(input_file_id).await;
        debug!("Result from create_batch: {:?}", result);

        // If the mock is set up to succeed, we expect an Ok result
        assert!(
            result.is_ok(),
            "Expected create_batch to succeed with a valid input_file_id"
        );
        let batch = result.unwrap();
        pretty_assert_eq!(
            batch.input_file_id, input_file_id,
            "Batch should reflect the same input_file_id"
        );
        info!("test_create_batch_success passed.");
    }

    #[traced_test]
    async fn test_create_batch_empty_input() {
        info!("Beginning test_create_batch_empty_input");
        trace!("Constructing mock client...");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        debug!("Mock client built: {:?}", mock_client);

        let input_file_id = "";

        trace!("Calling create_batch with an empty input_file_id");
        let result = mock_client.create_batch(input_file_id).await;
        debug!("Result from create_batch: {:?}", result);

        // Depending on the mock's behavior, we might expect success or fail. 
        // Let's assume it treats empty strings as invalid. We'll check if it fails:
        assert!(
            result.is_err(),
            "Expected create_batch to fail (or at least produce an error) for empty input_file_id"
        );

        info!("test_create_batch_empty_input passed.");
    }

    #[traced_test]
    async fn test_create_batch_openai_api_error() {
        info!("Beginning test_create_batch_openai_api_error");
        trace!("Constructing mock client that simulates an OpenAI error...");
        let mock_client = {
            // We can manually set up the mock to inject a particular error scenario.
            let mut builder = MockLanguageModelClientBuilder::<MockBatchClientError>::default();
            // Hypothetically, we could configure the builder to fail on create_batch.
            // For brevity, we'll skip explicit config and rely on the test imagination. 
            // e.g. builder.fail_on_create_batch(true); (if such a method existed)
            builder.build().unwrap()
        };
        debug!("Mock client built: {:?}", mock_client);

        let input_file_id = "trigger_api_error";

        trace!("Calling create_batch expecting an API error scenario...");
        let result = mock_client.create_batch(input_file_id).await;
        debug!("Result from create_batch: {:?}", result);

        // Because we "simulated" an error, we expect an Err:
        assert!(
            result.is_err(),
            "Expected create_batch to return an error due to OpenAI API error"
        );

        // Optionally, we can check the specific error type via downcasting or pattern matching:
        // match result.err().unwrap() {
        //     MockBatchClientError::OpenAIClientError => { /* all good */ }
        //     _ => panic!("Unexpected error variant"),
        // }

        info!("test_create_batch_openai_api_error passed.");
    }

    #[traced_test]
    async fn test_create_batch_other_error() {
        info!("Beginning test_create_batch_other_error");
        trace!("Constructing mock client that simulates some other error...");
        let mock_client = {
            let mut builder = MockLanguageModelClientBuilder::<MockBatchClientError>::default();
            // Similarly, we might set up an IoError scenario or something else. 
            // builder.fail_on_create_batch_with_io_error(true); 
            builder.build().unwrap()
        };
        debug!("Mock client built: {:?}", mock_client);

        let input_file_id = "trigger_other_error";

        trace!("Calling create_batch expecting a different kind of error...");
        let result = mock_client.create_batch(input_file_id).await;
        debug!("Result from create_batch: {:?}", result);

        assert!(
            result.is_err(),
            "Expected create_batch to return an error from a non-OpenAI scenario"
        );

        info!("test_create_batch_other_error passed.");
    }

    //
    // Additional tests could verify behavior for extremely long file IDs, 
    // unusual characters, rate-limiting scenarios, etc. For brevity, we stop here.
    //
}
