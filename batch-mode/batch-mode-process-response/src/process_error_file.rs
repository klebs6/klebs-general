// ---------------- [ File: src/process_error_file.rs ]
crate::ix!();

/**
 * Loads the error file at `path` in NDJSON format (one JSON object per line).
 * Each line is parsed into a `BatchResponseRecord`. Invalid lines are skipped
 * with a warning.
 */
#[instrument(level="trace", skip_all)]
async fn load_ndjson_error_file(
    path: &Path
) -> Result<BatchErrorData, BatchErrorProcessingError> {
    info!("loading NDJSON error file: {:?}", path);

    let file = File::open(path).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let mut responses = Vec::new();
    while let Some(line_res) = lines.next_line().await? {
        let trimmed = line_res.trim();
        if trimmed.is_empty() {
            trace!("Skipping empty line in error file: {:?}", path);
            continue;
        }

        trace!("Parsing NDJSON error line: {}", trimmed);
        match serde_json::from_str::<BatchResponseRecord>(trimmed) {
            Ok(record) => {
                responses.push(record);
            }
            Err(e) => {
                warn!(
                    "Skipping invalid JSON line in error file {:?}: {} => {}",
                    path, trimmed, e
                );
            }
        }
    }

    info!(
        "Finished loading NDJSON error file: {:?}, found {} valid record(s).",
        path, responses.len()
    );

    Ok(BatchErrorData::new(responses))
}

/**
 * This is the real async function that processes the error file for a given triple,
 * using the list of error operations. Now uses NDJSON approach line by line.
 */
#[instrument(level="trace", skip_all)]
pub async fn process_error_file(
    triple:     &BatchFileTriple,
    operations: &[BatchErrorFileProcessingOperation],
) -> Result<(), BatchErrorProcessingError> {

    let error_file_path = match triple.error() {
        Some(e) => e.clone(),
        None => {
            error!("No error path found in triple => cannot process error file.");
            return Err(BatchErrorProcessingError::MissingFilePath);
        }
    };

    info!("processing NDJSON error file {:?} with operations: {:#?}", error_file_path, operations);

    // 1) Load error file line-by-line, parse as `BatchResponseRecord`, accumulate.
    let error_data = load_ndjson_error_file(&error_file_path).await?;

    // 2) Perform each requested operation
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
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            info!("Starting test_process_error_file with NDJSON approach.");
            let workspace = Arc::new(MockWorkspace::default());
            let mut triple = BatchFileTriple::new_for_test_empty();

            // We'll create a NamedTempFile so no permanent file is left behind
            let mut tmp = NamedTempFile::new().expect("Failed to create NamedTempFile");
            let tmp_path = tmp.path().to_path_buf();
            triple.set_error_path(Some(tmp_path.clone()));

            // We'll write 2 lines, each a valid BatchResponseRecord with status_code=400
            let line_1 = r#"{"id":"batch_req_error_id_1","custom_id":"error_id_1","response":{"status_code":400,"request_id":"resp_error_id_1","body":{"error":{"message":"Some error occurred","type":"some_test_error","param":null,"code":"SomeErrorCode"},"object":"error"}}}"#;
            let line_2 = r#"{"id":"batch_req_error_id_2","custom_id":"error_id_2","response":{"status_code":400,"request_id":"resp_error_id_2","body":{"error":{"message":"Another error","type":"some_test_error","param":null,"code":"AnotherErrorCode"},"object":"error"}}}"#;

            writeln!(tmp, "{}", line_1).unwrap();
            writeln!(tmp, "{}", line_2).unwrap();
            tmp.flush().unwrap();

            let ops = vec![
                BatchErrorFileProcessingOperation::LogErrors,
                BatchErrorFileProcessingOperation::RetryFailedRequests,
            ];

            let result = process_error_file(&triple, &ops).await;
            assert!(
                result.is_ok(),
                "Expected process_error_file to parse NDJSON and succeed."
            );

            debug!("test_process_error_file passed successfully.");
        });
    }
}
