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
