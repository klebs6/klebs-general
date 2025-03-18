// ---------------- [ File: src/log_errors.rs ]
crate::ix!();

impl BatchFileTriple {

    pub async fn log_errors(&self, error_data: &BatchErrorData) 
        -> Result<(), BatchErrorProcessingError> 
    {
        info!("logging possible errors in our BatchErrorData of len {}", error_data.len());

        for response_record in error_data.responses() {
            if let BatchResponseBody::Error(error_body) = response_record.response().body() {
                let message = error_body.error().message();
                let custom_id = response_record.custom_id().as_str();
                println!("Error in request {}: {}", custom_id, message);
                // Replace with proper logging
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod batch_file_triple_log_errors_exhaustive_tests {
    use super::*;
    use serde_json::json;

    #[traced_test]
    async fn log_errors_handles_empty_error_data_gracefully() {
        trace!("===== BEGIN TEST: log_errors_handles_empty_error_data_gracefully =====");

        // Arrange
        let triple = make_mock_batch_file_triple();
        let empty_error_data = BatchErrorData::new(vec![]);

        // Act
        let res = triple.log_errors(&empty_error_data).await;
        debug!("Result of log_errors call on empty data: {:?}", res);

        // Assert
        assert!(res.is_ok(), "log_errors should succeed even with empty error data");

        trace!("===== END TEST: log_errors_handles_empty_error_data_gracefully =====");
    }

    #[traced_test]
    async fn log_errors_detects_and_logs_error_bodies() {
        trace!("===== BEGIN TEST: log_errors_detects_and_logs_error_bodies =====");

        // Arrange
        let triple = make_mock_batch_file_triple();

        // We explicitly create a 400 error record with "Some error message"
        let error_response = BatchResponseRecord::mock_with_code_and_body(
            "test_failing_item",
            400,
            &json!({
                "error": {
                    "message": "Some error message",
                    "type": "test_error",
                    "param": null,
                    "code": null
                }
            }),
        );
        let error_data = BatchErrorData::new(vec![error_response]);

        // Act
        let res = triple.log_errors(&error_data).await;
        debug!("Result of log_errors call on single error: {:?}", res);

        // We expect it to succeed but also to log the error lines.
        assert!(res.is_ok(), "log_errors should succeed with valid error data");

        trace!("===== END TEST: log_errors_detects_and_logs_error_bodies =====");
    }
}
