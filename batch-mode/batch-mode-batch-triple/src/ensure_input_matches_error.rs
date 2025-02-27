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
