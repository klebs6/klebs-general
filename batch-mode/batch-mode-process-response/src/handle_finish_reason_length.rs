// ---------------- [ File: batch-mode-process-response/src/handle_finish_reason_length.rs ]
crate::ix!();

pub async fn handle_finish_reason_length(
    failed_id:       &str,
    _message_content: &BatchMessageContent,
) -> Result<(), BatchSuccessResponseHandlingError> {

    /*
    // Adjust the prompt to be more concise
    let adjusted_prompt = adjust_prompt_for_retry(&original_request.messages[0].content);

    // Create a new request with adjusted prompt
    let retry_request = LanguageModelBatchAPIRequest {
        messages: vec![ChatCompletionRequestMessage {
            role: "user".to_string(),
            content: adjusted_prompt,
        }],
        max_tokens: Some(1000), // Adjust as needed
                                // ... other fields ...
    };

    // Send the retry request
    let retry_response = send_retry_request(retry_request).await?;

    // Handle the retry response recursively
    return handle_successful_response(&retry_response, &retry_request).await;
    */

    error!(
        "{}", 
        format!(
            "Response was truncated for request ID '{}'. We will extract what we can from the broken json string. We could eventually implement a way to retry here with adjusted query parameters",
            failed_id
        )
    );

    Ok(())
}

#[cfg(test)]
mod handle_finish_reason_length_tests {
    use super::*;
    use futures::executor::block_on;

    #[traced_test]
    async fn test_handle_finish_reason_length() {
        let message_content = BatchMessageContentBuilder::default()
            .content("Partial/truncated response".to_string()) // <-- REPLACED direct .from(...)
            .build()
            .unwrap();
        let result = handle_finish_reason_length("some_truncated_id", &message_content).await;
        assert!(result.is_ok());
    }
}
