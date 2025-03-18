// ---------------- [ File: src/ensure_input_matches_output.rs ]
crate::ix!();

impl BatchFileTriple {

    pub async fn ensure_input_matches_output(&self) 
        -> Result<(), BatchValidationError> 
    {
        // Load input and output files
        let input_data  = load_input_file(self.input().as_ref().unwrap()).await?;

        let output_data = load_output_file(self.output().as_ref().unwrap()).await?;

        // Compare request IDs
        let input_ids:  HashSet<_> = input_data.request_ids().into_iter().collect();
        let output_ids: HashSet<_> = output_data.request_ids().into_iter().collect();

        if input_ids != output_ids {
            return Err(BatchValidationError::RequestIdsMismatch {
                index:      self.index().clone(),
                input_ids:  Some(input_ids),
                output_ids: Some(output_ids),
                error_ids:  None,
            });
        }

        info!("for our batch triple {:#?}, we have now ensured the input request ids match the request ids from the output file",self);

        Ok(())
    }
}

#[cfg(test)]
mod batch_file_triple_ensure_input_matches_output_exhaustive_tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    use tokio::runtime::Runtime;
    use tracing::*;

    #[traced_test]
    fn ensure_input_matches_output_succeeds_with_identical_ids() {
        info!("Starting test: ensure_input_matches_output_succeeds_with_identical_ids");

        // Create the input file with 2 requests
        let mut input_file = NamedTempFile::new().expect("Failed to create a temp file for input");
        {
            // Each entire JSON object must be on exactly one line
            let req1 = LanguageModelBatchAPIRequest::mock("id-1");
            let req2 = LanguageModelBatchAPIRequest::mock("id-2");

            writeln!(input_file, "{}", serde_json::to_string(&req1).unwrap())
                .expect("Failed to write req1 to input file");
            writeln!(input_file, "{}", serde_json::to_string(&req2).unwrap())
                .expect("Failed to write req2 to input file");
        }

        // Create the output file with 2 response records, each on exactly ONE line.
        // Because the parser reads line by line, multi-line JSON objects will be broken.
        let mut output_file = NamedTempFile::new().expect("Failed to create a temp file for output");
        {
            let line_1 = r#"{"id":"batch_req_id-1","custom_id":"id-1","response":{"status_code":200,"request_id":"resp_req_id-1","body":{"id":"success-id","object":"chat.completion","created":0,"model":"test-model","choices":[],"usage":{"prompt_tokens":0,"completion_tokens":0,"total_tokens":0,"prompt_tokens_details":null,"completion_tokens_details":null},"system_fingerprint":null}},"error":null}"#;
            let line_2 = r#"{"id":"batch_req_id-2","custom_id":"id-2","response":{"status_code":200,"request_id":"resp_req_id-2","body":{"id":"success-id","object":"chat.completion","created":0,"model":"test-model","choices":[],"usage":{"prompt_tokens":0,"completion_tokens":0,"total_tokens":0,"prompt_tokens_details":null,"completion_tokens_details":null},"system_fingerprint":null}},"error":null}"#;

            writeln!(output_file, "{}", line_1)
                .expect("Failed to write line_1 to output file");
            writeln!(output_file, "{}", line_2)
                .expect("Failed to write line_2 to output file");
        }

        // Construct a triple referencing these new files
        let triple = BatchFileTriple::new_direct(
            &BatchIndex::Usize(2),
            Some(input_file.path().to_path_buf()),
            Some(output_file.path().to_path_buf()),
            None, // no error file for this test
            None,
            Arc::new(MockWorkspace::default()),
        );

        let rt = Runtime::new().expect("Failed to create tokio Runtime");
        let result = rt.block_on(async { triple.ensure_input_matches_output().await });

        debug!("Result of ensure_input_matches_output: {:?}", result);
        assert!(
            result.is_ok(),
            "Should succeed when input and output share the same IDs"
        );

        info!("Finished test: ensure_input_matches_output_succeeds_with_identical_ids");
    }
}
