// ---------------- [ File: src/ensure_input_matches_error.rs ]
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

    /// We fix this test so the input file has requests, and the error file has
    /// *failing* responses with the same custom IDs.
    #[traced_test]
    fn ensure_input_matches_error_succeeds_with_identical_ids() {
        info!("Starting test: ensure_input_matches_error_succeeds_with_identical_ids");

        let mut input_file = NamedTempFile::new()
            .expect("Failed to create a temp file for input");
        let mut error_file = NamedTempFile::new()
            .expect("Failed to create a temp file for error");

        // Write input lines (LanguageModelBatchAPIRequest).
        {
            let req1 = LanguageModelBatchAPIRequest::mock("id-a");
            let req2 = LanguageModelBatchAPIRequest::mock("id-b");
            writeln!(input_file, "{}", serde_json::to_string(&req1).unwrap()).unwrap();
            writeln!(input_file, "{}", serde_json::to_string(&req2).unwrap()).unwrap();
        }

        // Write error lines with code=400, matching custom IDs "id-a" and "id-b".
        {
            let err_rec1 = BatchResponseRecord::mock_with_code("id-a", 400);
            let err_rec2 = BatchResponseRecord::mock_with_code("id-b", 400);
            writeln!(error_file, "{}", serde_json::to_string(&err_rec1).unwrap()).unwrap();
            writeln!(error_file, "{}", serde_json::to_string(&err_rec2).unwrap()).unwrap();
        }

        let triple = BatchFileTriple::new_direct(
            &BatchIndex::Usize(6),
            Some(input_file.path().to_path_buf()),
            None,
            Some(error_file.path().to_path_buf()),
            None,
            Arc::new(MockWorkspace::default()),
        );

        let rt = Runtime::new().expect("Failed to create tokio runtime");
        let res = rt.block_on(async { triple.ensure_input_matches_error().await });

        debug!("Result: {:?}", res);
        assert!(res.is_ok(), "Should succeed for matching IDs in input vs error");

        info!("Finished test: ensure_input_matches_error_succeeds_with_identical_ids");
    }
}
