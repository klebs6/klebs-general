// ---------------- [ File: src/process_error_file.rs ]
crate::ix!();

/**
 * This is the real async function that processes the error file
 * for a given triple, using the list of error operations.
 */
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
        }
    }

    Ok(())
}

/**
 * The bridging function matching `ErrorFileFn`: 
 * 
 *  for<'a> fn(
 *      &'a BatchFileTriple,
 *      &'a [BatchErrorFileProcessingOperation],
 *  ) -> Pin<Box<dyn Future<Output=Result<(),BatchErrorProcessingError>> + Send + 'a>>
 */
fn process_error_file_bridge_fn<'a>(
    triple: &'a BatchFileTriple,
    ops: &'a [BatchErrorFileProcessingOperation],
) -> Pin<Box<dyn Future<Output = Result<(), BatchErrorProcessingError>> + Send + 'a>>
{
    Box::pin(async move {
        process_error_file(triple, ops).await
    })
}

/**
 * We expose a CONST of type `ErrorFileFn`, so passing `&PROCESS_ERROR_FILE_BRIDGE` 
 * exactly matches the trait's needed function pointer type.
 */
pub const PROCESS_ERROR_FILE_BRIDGE: ErrorFileFn = process_error_file_bridge_fn;
