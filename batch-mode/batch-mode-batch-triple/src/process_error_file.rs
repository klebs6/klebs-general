// ---------------- [ File: src/process_error_file.rs ]
crate::ix!();

impl BatchFileTriple {

    pub async fn log_errors(&self, error_data: &BatchErrorData) 
        -> Result<(), BatchErrorProcessingError> 
    {
        info!("logging possible errors in our BatchErrorData of len {}", error_data.len());

        for response_record in error_data.responses() {
            if let BatchResponseBody::Error(error_body) = response_record.response().body() {
                let message = error_body.error().message();
                let custom_id = response_record.custom_id().as_str();
                println!("Error in request {}: {}", custom_id, message);
                // Replace with proper logging
            }
        }
        Ok(())
    }

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
