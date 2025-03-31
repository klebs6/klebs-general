// ---------------- [ File: batch-mode-batch-triple/src/ensure_input_matches_error.rs ]
crate::ix!();

impl BatchFileTriple {

    pub async fn ensure_input_matches_error(&self) 
        -> Result<(), BatchValidationError> 
    {
        // Load input and error files
        let input_data = load_input_file(self.input().as_ref().unwrap()).await?;
        let error_data = load_error_file(self.error().as_ref().unwrap()).await?;

        // Compare request IDs
        let input_ids: HashSet<_> = input_data.request_ids().into_iter().collect();
        let error_ids: HashSet<_> = error_data.request_ids().into_iter().collect();

        if input_ids != error_ids {
            return Err(BatchValidationError::RequestIdsMismatch {
                index: self.index().clone(),
                input_ids:  Some(input_ids),
                output_ids: None,
                error_ids:  Some(error_ids),
            });
        }

        info!("for our batch triple {:#?}, we have now ensured the input request ids match the request ids from the error file", self);

        Ok(())
    }
}

#[cfg(test)]
mod batch_file_triple_ensure_input_matches_error_exhaustive_tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    use tokio::runtime::Runtime;
    use tracing::*;

    #[traced_test]
    fn ensure_input_matches_error_succeeds_with_identical_ids() {
        info!("Starting test: ensure_input_matches_error_succeeds_with_identical_ids");

        let mut input_file = NamedTempFile::new()
            .expect("Failed to create temp file for input");
        {
            // Each entire JSON object must be on exactly one line:
            let req_a = LanguageModelBatchAPIRequest::mock("id-a");
            let req_b = LanguageModelBatchAPIRequest::mock("id-b");

            writeln!(input_file, "{}", serde_json::to_string(&req_a).unwrap())
                .expect("Failed to write req_a");
            writeln!(input_file, "{}", serde_json::to_string(&req_b).unwrap())
                .expect("Failed to write req_b");
            }

        // code=400 => error scenario. Must match the shape for a single-line BatchResponseRecord
        // so that each line is parseable as a full JSON object
        let mut error_file = NamedTempFile::new()
            .expect("Failed to create temp file for error");
        {
            // Single-line JSON for the first record:
            let line_a = r#"{"id":"batch_req_id-a","custom_id":"id-a","response":{"status_code":400,"request_id":"resp_req_id-a","body":{"error":{"message":"Error for id-a","type":"test_error","param":null,"code":null}}},"error":null}"#;

            // Single-line JSON for the second record:
            let line_b = r#"{"id":"batch_req_id-b","custom_id":"id-b","response":{"status_code":400,"request_id":"resp_req_id-b","body":{"error":{"message":"Error for id-b","type":"test_error","param":null,"code":null}}},"error":null}"#;

            writeln!(error_file, "{}", line_a)
                .expect("Failed to write line_a to error file");
            writeln!(error_file, "{}", line_b)
                .expect("Failed to write line_b to error file");
            }

        let triple = BatchFileTriple::new_direct(
            &BatchIndex::Usize(6),
            Some(input_file.path().to_path_buf()),
            None,
            Some(error_file.path().to_path_buf()),
            None,
            Arc::new(MockBatchWorkspace::default()),
        );

        let rt = Runtime::new().expect("Failed to create tokio runtime");
        let res = rt.block_on(async { triple.ensure_input_matches_error().await });

        debug!("Result: {:?}", res);
        assert!(
            res.is_ok(),
            "Should succeed for matching IDs in input vs error"
        );

        info!("Finished test: ensure_input_matches_error_succeeds_with_identical_ids");
    }
}
