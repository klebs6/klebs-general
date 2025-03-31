// ---------------- [ File: batch-mode-batch-triple/src/retry_failed_requests.rs ]
crate::ix!();

impl BatchFileTriple {

    pub async fn retry_failed_requests(&self, error_data: &BatchErrorData) 
        -> Result<(), BatchErrorProcessingError> 
    {
        // Collect failed requests
        let _failed_requests = error_data.responses().iter().collect::<Vec<_>>();
        // Implement retry logic here
        //todo!("Implement retry logic for failed requests");
        Ok(())
    }
}

#[cfg(test)]
mod batch_file_triple_retry_failed_requests_exhaustive_tests {
    use super::*;
    use serde_json::json;

    #[traced_test]
    async fn retry_failed_requests_handles_empty_data() {
        trace!("===== BEGIN TEST: retry_failed_requests_handles_empty_data =====");
        let triple = make_mock_batch_file_triple();
        let empty_error_data = BatchErrorData::new(vec![]);
        let res = triple.retry_failed_requests(&empty_error_data).await;
        debug!("Result of retry_failed_requests on empty data: {:?}", res);
        assert!(res.is_ok(), "Should succeed with empty data");
        trace!("===== END TEST: retry_failed_requests_handles_empty_data =====");
    }

    #[traced_test]
    async fn retry_failed_requests_no_op_for_error_data() {
        trace!("===== BEGIN TEST: retry_failed_requests_no_op_for_error_data =====");
        let triple = make_mock_batch_file_triple();

        // We replace the old (BatchErrorBody::new / BatchResponse::new) calls
        // with an explicit mock_with_code_and_body that yields a 400 error record
        let error_response = BatchResponseRecord::mock_with_code_and_body(
            "req-99",
            400,
            &json!({
                "error": {
                    "message": "Retry me",
                    "type": "test_error",
                    "param": null,
                    "code": null
                }
            }),
        );
        let error_data = BatchErrorData::new(vec![error_response]);

        // Currently unimplemented; we just confirm it doesn't fail or panic.
        let res = triple.retry_failed_requests(&error_data).await;
        debug!("Result of retry_failed_requests call: {:?}", res);
        assert!(res.is_ok(), "Method is a no-op but should still succeed");
        trace!("===== END TEST: retry_failed_requests_no_op_for_error_data =====");
    }
}
