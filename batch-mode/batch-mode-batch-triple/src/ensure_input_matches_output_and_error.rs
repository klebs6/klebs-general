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

    /// We fix this so the input file has requests for "id-1" & "id-2",
    /// the output file has 2 success responses for "id-1" & "id-2",
    /// and the error file also has 2 *error* responses for "id-1" & "id-2".
    /// That way the union of output_ids & error_ids matches input_ids.
    #[traced_test]
    fn ensure_input_matches_output_and_error_succeeds_when_ids_match() {
        info!("Starting test: ensure_input_matches_output_and_error_succeeds_when_ids_match");

        let mut input_file = NamedTempFile::new().expect("Failed to create temp file for input");
        let mut output_file = NamedTempFile::new().expect("Failed to create temp file for output");
        let mut error_file = NamedTempFile::new().expect("Failed to create temp file for error");

        // Input: LanguageModelBatchAPIRequest for "id-1" and "id-2"
        {
            let req1 = LanguageModelBatchAPIRequest::mock("id-1");
            let req2 = LanguageModelBatchAPIRequest::mock("id-2");
            writeln!(input_file, "{}", serde_json::to_string(&req1).unwrap()).unwrap();
            writeln!(input_file, "{}", serde_json::to_string(&req2).unwrap()).unwrap();
        }

        // Output: two successes with code=200 for "id-1" and "id-2"
        {
            let rec1 = BatchResponseRecord::mock_with_code("id-1", 200);
            let rec2 = BatchResponseRecord::mock_with_code("id-2", 200);
            writeln!(output_file, "{}", serde_json::to_string(&rec1).unwrap()).unwrap();
            writeln!(output_file, "{}", serde_json::to_string(&rec2).unwrap()).unwrap();
        }

        // Error: (for demonstration, we'll say there *also* were error lines for "id-1" and "id-2")
        {
            let err1 = BatchResponseRecord::mock_with_code("id-1", 400);
            let err2 = BatchResponseRecord::mock_with_code("id-2", 400);
            writeln!(error_file, "{}", serde_json::to_string(&err1).unwrap()).unwrap();
            writeln!(error_file, "{}", serde_json::to_string(&err2).unwrap()).unwrap();
        }

        let triple = BatchFileTriple::new_direct(
            &BatchIndex::Usize(1),
            Some(input_file.path().to_path_buf()),
            Some(output_file.path().to_path_buf()),
            Some(error_file.path().to_path_buf()),
            None,
            Arc::new(MockWorkspace::default()),
        );

        let rt = Runtime::new().expect("Failed to create tokio runtime");
        let res = rt.block_on(async { triple.ensure_input_matches_output_and_error().await });

        debug!("Result of ensure_input_matches_output_and_error: {:?}", res);
        assert!(
            res.is_ok(),
            "Expected matching request IDs to succeed for input vs (output + error)"
        );

        info!("Finished test: ensure_input_matches_output_and_error_succeeds_when_ids_match");
    }
}
