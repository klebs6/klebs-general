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
 * The bridging function matching `BatchWorkflowProcessErrorFileFn`: 
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
 * We expose a CONST of type `BatchWorkflowProcessErrorFileFn`, so passing `&PROCESS_ERROR_FILE_BRIDGE` 
 * exactly matches the trait's needed function pointer type.
 */
pub const PROCESS_ERROR_FILE_BRIDGE: BatchWorkflowProcessErrorFileFn = process_error_file_bridge_fn;

#[cfg(test)]
mod process_error_file_tests {
    use super::*;

    #[traced_test]
    fn test_process_error_file() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let workspace = Arc::new(MockWorkspace::default());
            let mut triple = BatchFileTriple::new_for_test_empty();
            let error_file_name = "test_error.json";
            triple.set_error_path(Some(error_file_name.into()));

            let err_details = BatchErrorDetailsBuilder::default()
                .code("SomeErrorCode".to_string())
                .message("Some error occurred".to_string())
                .build()
                .unwrap();
            let err_body = BatchErrorResponseBodyBuilder::default()
                .error(err_details)
                .build()
                .unwrap();
            let response_content = BatchResponseContentBuilder::default()
                .status_code(400_u16)
                .request_id(ResponseRequestId::new("resp_error_id_1"))
                .body(BatchResponseBody::Error(err_body))
                .build()
                .unwrap();
            let record_error = BatchResponseRecordBuilder::default()
                .custom_id(CustomRequestId::new("error_id_1")) // <-- WORKS
                .response(response_content)
                .build()
                .unwrap();
            let error_data = BatchErrorDataBuilder::default()
                .responses(vec![record_error])
                .build()
                .unwrap();

            let serialized = serde_json::to_string_pretty(&error_data).unwrap();
            std::fs::write(error_file_name, &serialized).unwrap();

            let ops = vec![
                BatchErrorFileProcessingOperation::LogErrors,
                BatchErrorFileProcessingOperation::RetryFailedRequests,
            ];

            let result = process_error_file(&triple, &ops).await;
            assert!(result.is_ok());
            let _ = fs::remove_file(error_file_name);
        });
    }
}

