// ---------------- [ File: src/process_error_file.rs ]
crate::ix!();

pub async fn process_error_file(
    triple:     &BatchFileTriple,
    operations: &[BatchErrorFileProcessingOperation],
) -> Result<(), BatchErrorProcessingError> {

    let error_file = triple.error().as_ref().unwrap();

    info!("processing batch error file {:?} with operations: {:#?}", error_file, operations);

    let error_data = load_error_file(error_file).await?;

    for operation in operations {
        match operation {
            BatchErrorFileProcessingOperation::LogErrors => {
                triple.log_errors(&error_data).await?;
            }
            BatchErrorFileProcessingOperation::RetryFailedRequests => {
                triple.retry_failed_requests(&error_data).await?;
            }
            // Handle other operations
        }
    }

    Ok(())
}
