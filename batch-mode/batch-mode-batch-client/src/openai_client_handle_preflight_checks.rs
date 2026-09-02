// ---------------- [ File: batch-mode-batch-client/src/openai_client_handle_preflight_checks.rs ]
crate::ix!();

#[async_trait]
impl<E: Debug> PreflightCheckOpenAIApiKey
    for std::sync::Arc<dyn LanguageModelClientInterface<E>>
{
    type Error = E;

    async fn preflight_check_openai_api_key(&self) -> Result<(), Self::Error> {
        self.as_ref().preflight_check_openai_api_key().await
    }
}

#[async_trait]
impl<E> PreflightCheckOpenAIApiKey for OpenAIClientHandle<E>
where
    E: Debug + Send + Sync + From<OpenAIClientError>,
{
    type Error = E;

    async fn preflight_check_openai_api_key(&self) -> Result<(), Self::Error> {
        let timeout = std::time::Duration::from_secs(10);
        let start = std::time::Instant::now();

        info!(
            timeout_secs = timeout.as_secs(),
            "starting OpenAI API key preflight check via models.list()"
        );

        let client = self.client();
        let models = client.models();

        let list_future = models.list();
        let outcome = tokio::time::timeout(timeout, list_future).await;

        match outcome {
            Ok(Ok(model_list)) => {
                let elapsed = start.elapsed();
                info!(
                    model_count = model_list.data.len(),
                    elapsed_ms = elapsed.as_millis() as u64,
                    "OpenAI API key preflight check succeeded"
                );
                Ok(())
            }
            Ok(Err(openai_err)) => {
                let elapsed = start.elapsed();

                match &openai_err {
                    async_openai::error::OpenAIError::ApiError(api_err) => {
                        error!(
                            elapsed_ms = elapsed.as_millis() as u64,
                            message = %api_err.message,
                            error_type = ?api_err.r#type,
                            error_param = ?api_err.param,
                            error_code = ?api_err.code,
                            "OpenAI API key preflight check failed with API error"
                        );
                    }
                    _ => {
                        error!(
                            elapsed_ms = elapsed.as_millis() as u64,
                            error = %openai_err,
                            "OpenAI API key preflight check failed"
                        );
                    }
                }

                Err(E::from(OpenAIClientError::OpenAIError(openai_err)))
            }
            Err(_elapsed) => {
                let elapsed = start.elapsed();

                error!(
                    timeout_secs = timeout.as_secs(),
                    elapsed_ms = elapsed.as_millis() as u64,
                    "OpenAI API key preflight check timed out"
                );

                let api_err = OpenAIApiError {
                    message: format!(
                        "OpenAI API key preflight check timed out after {} seconds",
                        timeout.as_secs()
                    ),
                    r#type: None,
                    param: None,
                    code: None,
                };

                Err(E::from(OpenAIClientError::ApiError(api_err)))
            }
        }
    }
}

#[cfg(test)]
mod openai_api_key_preflight_interface_tests {
    use super::*;
    use std::sync::Arc;

    #[traced_test]
    async fn preflight_check_succeeds_for_default_mock_client() {
        info!("Starting preflight_check_succeeds_for_default_mock_client");

        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .expect("Failed to build mock client");

        let client: &dyn LanguageModelClientInterface<MockBatchClientError> = &mock_client;

        let result = client.preflight_check_openai_api_key().await;
        debug!("preflight result: {:?}", result);

        assert!(
            result.is_ok(),
            "Expected preflight to succeed for default mock client"
        );
    }

    #[traced_test]
    async fn preflight_check_fails_with_configured_openai_error() {
        info!("Starting preflight_check_fails_with_configured_openai_error");

        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .fail_on_preflight_openai_error(true)
            .build()
            .expect("Failed to build mock client");

        let client: &dyn LanguageModelClientInterface<MockBatchClientError> = &mock_client;

        let result = client.preflight_check_openai_api_key().await;
        debug!("preflight result: {:?}", result);

        assert!(
            result.is_err(),
            "Expected preflight to fail when configured to return an OpenAI error"
        );

        match result.err().expect("expected Err") {
            MockBatchClientError::OpenAIClientError(OpenAIClientError::ApiError(api_err)) => {
                debug!(
                    message = %api_err.message,
                    error_type = ?api_err.r#type,
                    error_param = ?api_err.param,
                    error_code = ?api_err.code,
                    "Observed expected OpenAI ApiError in preflight failure"
                );
            }
            other => {
                error!(?other, "Unexpected error variant from mock preflight");
                panic!("Unexpected error variant from mock preflight: {:?}", other);
            }
        }
    }

    #[traced_test]
    async fn preflight_check_fails_with_configured_other_error() {
        info!("Starting preflight_check_fails_with_configured_other_error");

        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .fail_on_preflight_other_error(true)
            .build()
            .expect("Failed to build mock client");

        let client: &dyn LanguageModelClientInterface<MockBatchClientError> = &mock_client;

        let result = client.preflight_check_openai_api_key().await;
        debug!("preflight result: {:?}", result);

        assert!(
            result.is_err(),
            "Expected preflight to fail when configured to return a non-OpenAI error"
        );

        match result.err().expect("expected Err") {
            MockBatchClientError::IoError(io_err) => {
                debug!(
                    kind = ?io_err.kind(),
                    "Observed expected IoError in preflight failure"
                );
            }
            other => {
                error!(?other, "Unexpected error variant from mock preflight");
                panic!("Unexpected error variant from mock preflight: {:?}", other);
            }
        }
    }

    #[traced_test]
    async fn preflight_check_is_callable_through_arc_dyn_language_model_client_interface() {
        info!("Starting preflight_check_is_callable_through_arc_dyn_language_model_client_interface");

        let mock_client = MockLanguageModelClientBuilder::<MockBatchClientError>::default()
            .build()
            .expect("Failed to build mock client");

        let mock_arc = Arc::new(mock_client);
        let client_arc: Arc<dyn LanguageModelClientInterface<MockBatchClientError>> = mock_arc;

        let result = client_arc.preflight_check_openai_api_key().await;
        debug!("preflight result: {:?}", result);

        assert!(
            result.is_ok(),
            "Expected preflight to be callable and succeed through Arc<dyn LanguageModelClientInterface>"
        );
    }
}
