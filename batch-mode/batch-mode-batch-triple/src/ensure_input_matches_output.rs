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

    /// We fix this test so that the input file has *requests* (LanguageModelBatchAPIRequest),
    /// and the output file has *responses* (BatchResponseRecord). We'll use matching custom IDs.
    #[traced_test]
    fn ensure_input_matches_output_succeeds_with_identical_ids() {
        info!("Starting test: ensure_input_matches_output_succeeds_with_identical_ids");

        let mut input_file = NamedTempFile::new()
            .expect("Failed to create a temp file for input");
        let mut output_file = NamedTempFile::new()
            .expect("Failed to create a temp file for output");

        // Write 2 requests with the same custom IDs we'll use in the output
        {
            let req1 = LanguageModelBatchAPIRequest::mock("id-1");
            let req2 = LanguageModelBatchAPIRequest::mock("id-2");
            let line_1 = serde_json::to_string(&req1).unwrap();
            let line_2 = serde_json::to_string(&req2).unwrap();
            writeln!(input_file, "{}", line_1).unwrap();
            writeln!(input_file, "{}", line_2).unwrap();
        }

        // Write 2 success response records with matching custom IDs
        {
            let rec1 = BatchResponseRecord::mock_with_code("id-1", 200);
            let rec2 = BatchResponseRecord::mock_with_code("id-2", 200);
            let line_1 = serde_json::to_string(&rec1).unwrap();
            let line_2 = serde_json::to_string(&rec2).unwrap();
            writeln!(output_file, "{}", line_1).unwrap();
            writeln!(output_file, "{}", line_2).unwrap();
        }

        let triple = BatchFileTriple::new_direct(
            &BatchIndex::Usize(2),
            Some(input_file.path().to_path_buf()),
            Some(output_file.path().to_path_buf()),
            None,
            None,
            Arc::new(MockWorkspace::default()),
        );

        let rt = Runtime::new().expect("Failed to create tokio Runtime");
        let result = rt.block_on(async { triple.ensure_input_matches_output().await });

        debug!("Result of ensure_input_matches_output: {:?}", result);
        assert!(result.is_ok(), "Should succeed when input and output share the same IDs");

        info!("Finished test: ensure_input_matches_output_succeeds_with_identical_ids");
    }
}
