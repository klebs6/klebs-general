crate::ix!();

#[async_trait]
impl<E> WaitForBatchCompletion for OpenAIClientHandle<E>
where
    E: Debug + Send + Sync + From<OpenAIClientError>
{
    type Error = E;

    async fn wait_for_batch_completion(&self, batch_id: &str)
        -> Result<Batch, Self::Error>
    {
        info!("waiting for batch completion: batch_id={}", batch_id);

        loop {
            let batch = self.retrieve_batch(batch_id).await?;

            match batch.status {
                BatchStatus::Completed => return Ok(batch),
                BatchStatus::Failed => {
                    // Return an error: 
                    let openai_err = OpenAIClientError::ApiError(OpenAIApiError {
                        message: "Batch failed".to_owned(),
                        r#type: None,
                        param:  None,
                        code:   None,
                    });
                    return Err(E::from(openai_err));
                }
                _ => {
                    println!("Batch status: {:?}", batch.status);
                    tokio::time::sleep(std::time::Duration::from_secs(20)).await;
                }
            }
        }
    }
}

#[cfg(test)]
mod wait_for_batch_completion_tests {
    use super::*;
    use futures::executor::block_on;
    use std::sync::Arc;
    use tracing::{debug, error, info, trace, warn};

    #[traced_test]
    async fn test_wait_for_batch_completion_immediate_success() {
        info!("Beginning test_wait_for_batch_completion_immediate_success");
        trace!("Constructing mock client that immediately returns a completed batch...");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        let mock_client = {
            let c = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
                .build()
                .unwrap();
            // Make the batch "immediate_success_id" be completed from the start:
            {
                let mut guard = c.batches().write().unwrap();
                guard.insert(
                    "immediate_success_id".to_string(),
                    Batch {
                        id:                 "immediate_success_id".to_string(),
                        status:             BatchStatus::Completed,
                        input_file_id:      "some_file".to_string(),
                        completion_window:  "24h".to_string(),
                        object:             "batch".to_string(),
                        endpoint:           "/v1/chat/completions".to_string(),
                        errors:             None,
                        output_file_id:     None,
                        error_file_id:      None,
                        created_at:         0,
                        in_progress_at:     None,
                        expires_at:         None,
                        finalizing_at:      None,
                        completed_at:       None,
                        failed_at:          None,
                        expired_at:         None,
                        cancelling_at:      None,
                        cancelled_at:       None,
                        request_counts:     None,
                        metadata:           None,
                    },
                );
            }
            c
        };

        debug!("Mock client built: {:?}", mock_client);

        let batch_id = "immediate_success_id";

        trace!("Calling wait_for_batch_completion on mock_client with batch_id={}", batch_id);
        let result = mock_client.wait_for_batch_completion(batch_id).await;
        debug!("Result from wait_for_batch_completion: {:?}", result);

        // "immediate_success_id" is forcibly set to Completed on first retrieval
        assert!(
            result.is_ok(),
            "Expected wait_for_batch_completion to succeed if the batch is already completed"
        );
        let batch = result.unwrap();
        pretty_assert_eq!(
            batch.status,
            BatchStatus::Completed,
            "Batch status should be Completed"
        );
        info!("test_wait_for_batch_completion_immediate_success passed.");
    }

    #[traced_test]
    async fn test_wait_for_batch_completion_immediate_failure() {
        info!("Beginning test_wait_for_batch_completion_immediate_failure");
        trace!("Constructing mock client that immediately returns a failed batch...");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        debug!("Mock client built: {:?}", mock_client);

        let batch_id = "immediate_failure_id";

        trace!("Calling wait_for_batch_completion on mock_client with batch_id={}", batch_id);
        let result = mock_client.wait_for_batch_completion(batch_id).await;
        debug!("Result from wait_for_batch_completion: {:?}", result);

        // Because "immediate_failure_id" is forcibly set to Failed on first retrieve,
        // we expect an error.
        assert!(
            result.is_err(),
            "Expected wait_for_batch_completion to return error if the batch is failed at once"
        );
        info!("test_wait_for_batch_completion_immediate_failure passed.");
    }

    #[traced_test]
    async fn test_wait_for_batch_completion_eventual_failure() {
        info!("Beginning test_wait_for_batch_completion_eventual_failure");
        trace!("Constructing mock client that simulates in-progress followed by failure...");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        debug!("Mock client built: {:?}", mock_client);

        let batch_id = "eventual_failure_id";

        trace!("Calling wait_for_batch_completion expecting multiple in-progress checks before failure");
        let result = mock_client.wait_for_batch_completion(batch_id).await;
        debug!("Result from wait_for_batch_completion: {:?}", result);

        // Because the retrieve logic toggles from InProgress -> Failed,
        // we eventually get a failure. So we expect an Err.
        assert!(
            result.is_err(),
            "Expected wait_for_batch_completion to error after an eventual failure status"
        );
        info!("test_wait_for_batch_completion_eventual_failure passed.");
    }

    #[traced_test]
    async fn test_wait_for_batch_completion_openai_error() {
        info!("Beginning test_wait_for_batch_completion_openai_error");
        trace!("Constructing mock client that simulates an OpenAI error during retrieve_batch...");
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();
        debug!("Mock client built: {:?}", mock_client);

        let batch_id = "trigger_api_error";

        trace!("Calling wait_for_batch_completion expecting an OpenAI error from retrieve_batch");
        let result = mock_client.wait_for_batch_completion(batch_id).await;
        debug!("Result from wait_for_batch_completion: {:?}", result);

        // Because "trigger_api_error" forcibly returns an OpenAI error on the first retrieve,
        // we expect an Err from wait_for_batch_completion
        assert!(
            result.is_err(),
            "Expected wait_for_batch_completion to fail due to an OpenAI error in retrieve_batch"
        );
        info!("test_wait_for_batch_completion_openai_error passed.");
    }

    #[traced_test]
    async fn test_wait_for_batch_completion_eventual_success() {
        info!("Beginning test_wait_for_batch_completion_eventual_success");

        // Build the mock
        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .unwrap();

        // So that the batch "eventual_success_id" is InProgress initially, 
        // but flips to Completed on the FIRST retrieval (or second— you decide):
        mock_client.configure_inprogress_then_complete_with("eventual_success_id", /*want_output=*/false, /*want_error=*/false);

        info!("Calling wait_for_batch_completion expecting multiple in-progress checks before completion");
        let result = mock_client.wait_for_batch_completion("eventual_success_id").await;
        debug!("Result from wait_for_batch_completion: {:?}", result);

        assert!(
            result.is_ok(),
            "Expected wait_for_batch_completion to succeed after in-progress statuses"
        );
        let final_batch = result.unwrap();
        pretty_assert_eq!(final_batch.status, BatchStatus::Completed);
        info!("test_wait_for_batch_completion_eventual_success passed.");
    }
}
