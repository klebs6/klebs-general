// ---------------- [ File: src/load_batch_files.rs ]
crate::ix!();

/**
 * ALL-OR-NOTHING loader for the input file.  
 * 
 * Reads NDJSON (one `LanguageModelBatchAPIRequest` per line). 
 * If **any** line is invalid, we immediately fail (do not skip).
 */
pub async fn load_input_file(path: impl AsRef<Path>) -> Result<BatchInputData, JsonParseError> {
    info!("loading input file: {:?}", path.as_ref());

    // If the file doesn’t exist:
    if !path.as_ref().exists() {
        if is_test_mode() {
            warn!(
                "Mock scenario (test-only): Input file not found at {:?}; returning empty BatchInputData.",
                path.as_ref()
            );
            return Ok(BatchInputData::new(vec![]));
        } else {
            error!(
                "Input file does not exist at {:?}; failing with IoError",
                path.as_ref()
            );
            let io_err = std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Input file not found: {}", path.as_ref().display()),
            );
            return Err(JsonParseError::IoError(io_err));
        }
    }

    let file = File::open(path.as_ref()).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let mut requests = Vec::new();
    while let Some(line) = lines.next_line().await? {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            trace!("Skipping empty line in input file: {}", path.as_ref().display());
            continue;
        }

        trace!("Attempting to parse input line: {}", trimmed);
        // ALL OR NOTHING => if parse fails, return an error immediately.
        let request = match serde_json::from_str::<LanguageModelBatchAPIRequest>(trimmed) {
            Ok(req) => req,
            Err(e) => {
                error!(
                    "Invalid line in input file => returning error. line='{}' error='{}'",
                    trimmed, e
                );
                return Err(JsonParseError::SerdeError(e));
            }
        };
        requests.push(request);
    }

    info!(
        "Successfully loaded {} request(s) from input file: {:?}",
        requests.len(),
        path.as_ref()
    );
    Ok(BatchInputData::new(requests))
}

/**
 * ALL-OR-NOTHING loader for the error file.  
 * 
 * Reads NDJSON (one `BatchResponseRecord` per line). 
 * If any line is invalid, we return an error. No skipping.
 */
pub async fn load_error_file(path: impl AsRef<Path>) -> Result<BatchErrorData, JsonParseError> {
    info!("loading error file: {:?}", path.as_ref());

    if !path.as_ref().exists() {
        if is_test_mode() {
            warn!(
                "Mock scenario (test-only): Error file not found at {:?}; returning empty BatchErrorData.",
                path.as_ref()
            );
            return Ok(BatchErrorData::new(vec![]));
        } else {
            error!(
                "Error file does not exist at {:?}; failing with IoError",
                path.as_ref()
            );
            let io_err = std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Error file not found: {}", path.as_ref().display()),
            );
            return Err(JsonParseError::IoError(io_err));
        }
    }

    let file = File::open(path.as_ref()).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let mut responses = Vec::new();
    while let Some(line) = lines.next_line().await? {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            trace!("Skipping empty line in error file: {}", path.as_ref().display());
            continue;
        }

        trace!("Attempting to parse error-file line: {}", trimmed);
        // ALL OR NOTHING => if parse fails, return an error
        let response_record = match serde_json::from_str::<BatchResponseRecord>(trimmed) {
            Ok(r) => r,
            Err(e) => {
                error!(
                    "Invalid line in error file => returning error. line='{}' error='{}'",
                    trimmed, e
                );
                return Err(JsonParseError::SerdeError(e));
            }
        };
        responses.push(response_record);
    }

    info!(
        "Successfully loaded {} record(s) from error file: {:?}",
        responses.len(),
        path.as_ref()
    );
    Ok(BatchErrorData::new(responses))
}

/**
 * ALL-OR-NOTHING loader for the output file.  
 * 
 * Reads NDJSON (one `BatchResponseRecord` per line). 
 * If any line is invalid, we fail immediately. 
 */
pub async fn load_output_file(path: impl AsRef<Path>) -> Result<BatchOutputData, JsonParseError> {
    info!("loading output file: {:?}", path.as_ref());

    if !path.as_ref().exists() {
        if is_test_mode() {
            warn!(
                "Mock scenario (test-only): Output file not found at {:?}; returning empty BatchOutputData.",
                path.as_ref()
            );
            return Ok(BatchOutputData::new(vec![]));
        } else {
            error!(
                "Output file does not exist at {:?}; failing with IoError",
                path.as_ref()
            );
            let io_err = std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Output file not found: {}", path.as_ref().display()),
            );
            return Err(JsonParseError::IoError(io_err));
        }
    }

    let file = File::open(path.as_ref()).await?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let mut responses = Vec::new();
    while let Some(line) = lines.next_line().await? {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            trace!("Skipping empty line in output file: {}", path.as_ref().display());
            continue;
        }

        trace!("Attempting to parse output-file line: {}", trimmed);
        // ALL OR NOTHING => if parse fails, return an error
        let response_record = match serde_json::from_str::<BatchResponseRecord>(trimmed) {
            Ok(r) => r,
            Err(e) => {
                error!(
                    "Invalid line in output file => returning error. line='{}' error='{}'",
                    trimmed, e
                );
                return Err(JsonParseError::SerdeError(e));
            }
        };
        responses.push(response_record);
    }

    info!(
        "Successfully loaded {} record(s) from output file: {:?}",
        responses.len(),
        path.as_ref()
    );
    Ok(BatchOutputData::new(responses))
}

#[cfg(test)]
mod file_loading_tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    

    fn write_lines_to_temp_file(file: &mut NamedTempFile, lines: &[&str]) {
        for line in lines {
            writeln!(file, "{}", line).unwrap();
        }
    }

    #[traced_test]
    async fn should_load_input_file_successfully() {
        info!("Testing load_input_file with valid JSON lines.");

        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file for input test.");
        let line_1 = make_valid_lmb_api_request_json_mock("input-1");
        let line_2 = make_valid_lmb_api_request_json_mock("input-2");
        write_lines_to_temp_file(&mut temp_file, &[&line_1, &line_2]);

        let result = load_input_file(temp_file.path()).await;

        assert!(result.is_ok(), "Should successfully load valid input file.");
        let data = result.unwrap();
        pretty_assert_eq!(data.requests().len(), 2, "Should have exactly two requests.");
        debug!("Loaded {} requests from input file.", data.requests().len());
    }

    #[traced_test]
    async fn should_fail_load_input_file_with_invalid_json() {
        info!("Testing load_input_file with an invalid JSON line.");

        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file.");
        write_lines_to_temp_file(&mut temp_file, &["{ invalid json }"]);

        let result = load_input_file(temp_file.path()).await;

        assert!(result.is_err(), "Should fail to load invalid JSON line.");
        error!("Received expected error for malformed input JSON: {:?}", result.err());
    }

    #[traced_test]
    async fn should_handle_empty_file_for_input_load() {
        info!("Testing load_input_file with an empty file.");

        let temp_file = NamedTempFile::new().expect("Failed to create temp file.");

        let result = load_input_file(temp_file.path()).await;

        assert!(result.is_ok(), "Empty file should load successfully, returning 0 requests.");
        let data = result.unwrap();
        pretty_assert_eq!(data.requests().len(), 0, "Should have zero requests from empty file.");
        debug!("Empty file loaded without error, as expected.");
    }

    #[traced_test]
    async fn should_fail_load_error_file_with_invalid_json() {
        info!("Testing load_error_file with an invalid JSON line.");

        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file.");
        write_lines_to_temp_file(&mut temp_file, &["{{ broken stuff }}"]);

        let result = load_error_file(temp_file.path()).await;

        assert!(result.is_err(), "Should fail when encountering invalid JSON in error file.");
        warn!("Got expected parse error: {:?}", result.err());
    }

    #[traced_test]
    async fn should_handle_empty_file_for_error_load() {
        info!("Testing load_error_file with an empty file.");

        let temp_file = NamedTempFile::new().expect("Failed to create temp file.");

        let result = load_error_file(temp_file.path()).await;

        assert!(result.is_ok(), "Empty error file should load successfully, returning 0 responses.");
        let data = result.unwrap();
        pretty_assert_eq!(data.responses().len(), 0, "No responses expected from empty error file.");
        debug!("Empty error file loaded with no issues.");
    }

    #[traced_test]
    async fn should_fail_load_output_file_with_invalid_json() {
        info!("Testing load_output_file with invalid JSON line.");

        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file.");
        write_lines_to_temp_file(&mut temp_file, &["}{ definitely not valid JSON"]);

        let result = load_output_file(temp_file.path()).await;

        assert!(result.is_err(), "Should fail for malformed JSON lines.");
        error!("Encountered expected error: {:?}", result.err());
    }

    #[traced_test]
    async fn should_handle_empty_file_for_output_load() {
        info!("Testing load_output_file with an empty file.");

        let temp_file = NamedTempFile::new().expect("Failed to create temp file.");

        let result = load_output_file(temp_file.path()).await;

        assert!(result.is_ok(), "Should succeed loading an empty output file.");
        let data = result.unwrap();
        pretty_assert_eq!(data.responses().len(), 0, "No responses in empty file.");
        trace!("Empty file loaded correctly for output load.");
    }

    #[traced_test]
    async fn should_load_error_file_successfully() {
        info!("Testing load_error_file with valid NDJSON lines.");

        // Single-line JSON 1 (status_code=400; 'object':'error').
        let line_1 = r#"{"id":"batch_req_error-file-1","custom_id":"error-file-1","response":{"status_code":400,"request_id":"resp_req_error-file-1","body":{"error":{"message":"Error for error-file-1","type":"test_error","param":null,"code":null},"object":"error"}},"error":null}"#;

        // Single-line JSON 2 (same shape).
        let line_2 = r#"{"id":"batch_req_error-file-2","custom_id":"error-file-2","response":{"status_code":400,"request_id":"resp_req_error-file-2","body":{"error":{"message":"Error for error-file-2","type":"test_error","param":null,"code":null},"object":"error"}},"error":null}"#;

        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file for error file test.");
        writeln!(temp_file, "{}", line_1).expect("Failed to write line_1");
        writeln!(temp_file, "{}", line_2).expect("Failed to write line_2");

        let result = load_error_file(temp_file.path()).await;

        assert!(result.is_ok(), "Should successfully load valid error file lines.");
        let data = result.unwrap();
        pretty_assert_eq!(data.responses().len(), 2, "Should have exactly two responses for error data.");
        trace!("Loaded {} responses from error file.", data.responses().len());
    }

    #[traced_test]
    async fn should_load_output_file_successfully() {
        info!("Testing load_output_file with valid NDJSON lines.");

        // Single-line JSON 1 (status_code=200, 'object':'chat.completion').
        let line_1 = r#"{"id":"batch_req_output-file-1","custom_id":"output-file-1","response":{"status_code":200,"request_id":"resp_req_output-file-1","body":{"id":"success-id","object":"chat.completion","created":0,"model":"test-model","choices":[],"usage":{"prompt_tokens":0,"completion_tokens":0,"total_tokens":0}}},"error":null}"#;

        // Single-line JSON 2 (another success record).
        let line_2 = r#"{"id":"batch_req_output-file-2","custom_id":"output-file-2","response":{"status_code":200,"request_id":"resp_req_output-file-2","body":{"id":"success-id-2","object":"chat.completion","created":0,"model":"test-model-2","choices":[],"usage":{"prompt_tokens":10,"completion_tokens":20,"total_tokens":30}}},"error":null}"#;

        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file for output file test.");
        writeln!(temp_file, "{}", line_1).expect("Failed to write line_1");
        writeln!(temp_file, "{}", line_2).expect("Failed to write line_2");

        let result = load_output_file(temp_file.path()).await;

        assert!(result.is_ok(), "Should load valid output file lines successfully.");
        let data = result.unwrap();
        pretty_assert_eq!(data.responses().len(), 2, "Should have exactly 2 response records from output file.");
        debug!("Loaded {} records from output file.", data.responses().len());
    }
}
