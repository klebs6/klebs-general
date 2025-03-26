// ---------------- [ File: src/ensure_input_matches_output_and_error.rs ]
crate::ix!();

impl BatchFileTriple {

    pub async fn ensure_input_matches_output_and_error(
        &self,
    ) -> Result<(), BatchValidationError> {
        let input_data  = load_input_file(self.input().as_ref().unwrap()).await?;
        let output_data = load_output_file(self.output().as_ref().unwrap()).await?;
        let error_data  = load_error_file(self.error().as_ref().unwrap()).await?;

        let input_ids:  HashSet<_> = input_data.request_ids().into_iter().collect();
        let output_ids: HashSet<_> = output_data.request_ids().into_iter().collect();
        let error_ids:  HashSet<_> = error_data.request_ids().into_iter().collect();

        let combined_ids: HashSet<_> = output_ids.union(&error_ids).cloned().collect();

        if input_ids != combined_ids {
            return Err(BatchValidationError::RequestIdsMismatch {
                index:      self.index().clone(),
                input_ids:  Some(input_ids),
                output_ids: Some(output_ids),
                error_ids:  Some(error_ids),
            });
        }

        info!("for our batch triple {:#?}, we have now ensured the input request ids match the combined request ids from the output and error files",self);

        Ok(())
    }
}

#[cfg(test)]
mod batch_file_triple_ensure_input_matches_output_and_error_exhaustive_tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    use tokio::runtime::Runtime;
    use tracing::*;

    #[traced_test]
    fn ensure_input_matches_output_and_error_succeeds_when_ids_match() {
        info!("Starting test: ensure_input_matches_output_and_error_succeeds_when_ids_match");

        // Input with 2 requests
        let mut input_file = NamedTempFile::new().expect("Failed to create temp file for input");
        {
            let req1 = LanguageModelBatchAPIRequest::mock("id-1");
            let req2 = LanguageModelBatchAPIRequest::mock("id-2");

            writeln!(input_file, "{}", serde_json::to_string(&req1).unwrap())
                .expect("Failed to write req1");
            writeln!(input_file, "{}", serde_json::to_string(&req2).unwrap())
                .expect("Failed to write req2");
        }

        // Output file, code=200 => success lines, each as a single line
        let mut output_file = NamedTempFile::new().expect("Failed to create temp file for output");
        {
            let line_1 = r#"{"id":"batch_req_id-1","custom_id":"id-1","response":{"status_code":200,"request_id":"resp_req_id-1","body":{"id":"success-id","object":"chat.completion","created":0,"model":"test-model","choices":[],"usage":{"prompt_tokens":0,"completion_tokens":0,"total_tokens":0,"prompt_tokens_details":null,"completion_tokens_details":null},"system_fingerprint":null}},"error":null}"#;
            let line_2 = r#"{"id":"batch_req_id-2","custom_id":"id-2","response":{"status_code":200,"request_id":"resp_req_id-2","body":{"id":"success-id","object":"chat.completion","created":0,"model":"test-model","choices":[],"usage":{"prompt_tokens":0,"completion_tokens":0,"total_tokens":0,"prompt_tokens_details":null,"completion_tokens_details":null},"system_fingerprint":null}},"error":null}"#;

            writeln!(output_file, "{}", line_1)
                .expect("Failed to write line_1 to output file");
            writeln!(output_file, "{}", line_2)
                .expect("Failed to write line_2 to output file");
        }

        // Error file, code=400 => error lines, each as a single line
        let mut error_file = NamedTempFile::new().expect("Failed to create temp file for error");
        {
            let err_line_1 = r#"{"id":"batch_req_id-1","custom_id":"id-1","response":{"status_code":400,"request_id":"resp_req_id-1","body":{"error":{"message":"Error for id-1","type":"test_error","param":null,"code":null}}},"error":null}"#;
            let err_line_2 = r#"{"id":"batch_req_id-2","custom_id":"id-2","response":{"status_code":400,"request_id":"resp_req_id-2","body":{"error":{"message":"Error for id-2","type":"test_error","param":null,"code":null}}},"error":null}"#;

            writeln!(error_file, "{}", err_line_1)
                .expect("Failed to write err_line_1 to error file");
            writeln!(error_file, "{}", err_line_2)
                .expect("Failed to write err_line_2 to error file");
        }

        let triple = BatchFileTriple::new_direct(
            &BatchIndex::Usize(1),
            Some(input_file.path().to_path_buf()),
            Some(output_file.path().to_path_buf()),
            Some(error_file.path().to_path_buf()),
            None,
            Arc::new(MockBatchWorkspace::default()),
        );

        let rt = Runtime::new().expect("Failed to create tokio runtime");
        let res = rt
            .block_on(async { triple.ensure_input_matches_output_and_error().await });

        debug!(
            "Result of ensure_input_matches_output_and_error: {:?}",
            res
        );
        assert!(
            res.is_ok(),
            "Expected matching request IDs to succeed for input vs (output + error)"
        );

        info!("Finished test: ensure_input_matches_output_and_error_succeeds_when_ids_match");
    }
}
