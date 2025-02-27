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
