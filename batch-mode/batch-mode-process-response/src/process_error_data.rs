// ---------------- [ File: src/process_error_data.rs ]
crate::ix!();

pub async fn process_error_data(error_data: &BatchErrorData) 
-> Result<(), BatchErrorProcessingError> 
{
    for response_record in error_data.responses() {
        if let Some(error_body) = response_record.response().body().as_error() {
            eprintln!(
                "Error for Custom ID '{}': Code: {:?}, Message: {}",
                response_record.custom_id(),
                error_body.error().code(),
                error_body.error().message()
            );
            // Handle the error as needed
        }
    }
    Ok(())
}
