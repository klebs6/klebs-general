// ---------------- [ File: batch-mode-process-response/src/process_output_file.rs ]
crate::ix!();

/**
 * Loads the output file at `path` in NDJSON format (one JSON object per line).
 * Each line is parsed into a `BatchResponseRecord`. Invalid lines are skipped
 * with a warning, so that partial data doesn't abort the entire read.
 */
#[instrument(level="trace", skip_all)]
async fn load_ndjson_output_file(
    path: &Path
) -> Result<BatchOutputData, BatchOutputProcessingError> {
    info!("loading NDJSON output file: {:?}", path);

    // Attempt to open the file
    let file = File::open(path).await.map_err(BatchOutputProcessingError::from)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let mut responses = Vec::new();
    while let Some(line_res) = lines.next_line().await? {
        let trimmed = line_res.trim();
        if trimmed.is_empty() {
            trace!("Skipping empty line in output file: {:?}", path);
            continue;
        }

        trace!("Parsing NDJSON output line: {}", trimmed);
        match serde_json::from_str::<BatchResponseRecord>(trimmed) {
            Ok(record) => {
                responses.push(record);
            }
            Err(e) => {
                warn!(
                    "Skipping invalid JSON line in output file {:?}: {} => {}",
                    path, trimmed, e
                );
                // We skip this line instead of failing the entire load
            }
        }
    }

    info!(
        "Finished loading NDJSON output file: {:?}, found {} valid record(s).",
        path, responses.len()
    );

    Ok(BatchOutputData::new(responses))
}

/**
 * The core async function to process the output file for a given triple.
 * Now we do NDJSON line-by-line parsing, just like the older batch-mode approach.
 */
#[instrument(level="trace", skip_all)]
pub async fn process_output_file<T>(
    triple:                &BatchFileTriple,
    workspace:             &dyn BatchWorkspaceInterface,
    expected_content_type: &ExpectedContentType,
) -> Result<(), BatchOutputProcessingError> 
where
    T: 'static + Send + Sync + DeserializeOwned + Named + GetTargetPathForAIExpansion,
{
    trace!("process_output_file => index = {:?}", triple.index());

    // 1) Identify the path to the output file, which must be NDJSON (one record per line).
    let output_path = match triple.output() {
        Some(p) => p.clone(),
        None => {
            error!("No output path found in triple => cannot process output file.");
            return Err(BatchOutputProcessingError::MissingFilePath);
        }
    };

    // 2) Load as NDJSON => gather into a BatchOutputData
    let output_data = load_ndjson_output_file(&output_path).await?;

    // 3) Process all records
    process_output_data::<T>(&output_data, workspace, expected_content_type).await
}

/**
 * The bridging function EXACTLY matches the `BatchWorkflowProcessOutputFileFn` type:
 * 
 *   for<'a> fn(
 *       &'a BatchFileTriple,
 *       &'a (dyn BatchWorkspaceInterface + 'a),
 *       &'a ExpectedContentType,
 *   ) -> Pin<Box<dyn Future<Output=Result<(),BatchOutputProcessingError>> + Send + 'a>>
 */
pub fn process_output_file_bridge_fn<'a, T>(
    triple:    &'a BatchFileTriple,
    workspace: &'a (dyn BatchWorkspaceInterface + 'a),
    ect:       &'a ExpectedContentType,
) -> Pin<Box<dyn Future<Output = Result<(), BatchOutputProcessingError>> + Send + 'a>> 
where
    T: 'static + Send + Sync + DeserializeOwned + Named + GetTargetPathForAIExpansion,
{
    Box::pin(async move {
        process_output_file::<T>(triple, workspace, ect).await
    })
}

/**
 * A default bridging function, if the user doesn't specify a particular T type.
 * We'll parse each line as a `CamelCaseTokenWithComment` or whichever default type we prefer.
 */
pub fn default_output_file_bridge_fn<'a>(
    triple:    &'a BatchFileTriple,
    workspace: &'a (dyn BatchWorkspaceInterface + 'a),
    ect:       &'a ExpectedContentType,
) -> Pin<Box<dyn Future<Output = Result<(), BatchOutputProcessingError>> + Send + 'a>>
{
    Box::pin(async move {
        // Use CamelCaseTokenWithComment (or any other default T) as the type:
        process_output_file::<CamelCaseTokenWithComment>(triple, workspace, ect).await
    })
}

/**
 * The const pointer the macro references.
 */
pub const DEFAULT_OUTPUT_FILE_BRIDGE: BatchWorkflowProcessOutputFileFn 
    = default_output_file_bridge_fn;


#[cfg(test)]
mod process_output_file_tests {
    use super::*;

    #[derive(Debug, Clone, Deserialize, Serialize, NamedItem)]
    pub struct OutputFileMockItem {
        pub name: String,
    }

    #[traced_test]
    async fn test_process_output_file_ok() {
        info!("Starting test_process_output_file_ok with NDJSON approach.");

        let workspace: Arc<dyn BatchWorkspaceInterface> = BatchWorkspace::new_temp().await.unwrap();

        let mut triple = BatchFileTriple::new_for_test_with_workspace(workspace.clone());
        triple.set_input_path(Some("dummy_input.json".into()));

        // We create a NamedTempFile to avoid littering permanent files:
        let mut tmpfile = NamedTempFile::new().expect("Failed to create NamedTempFile");
        let tmp_path = tmpfile.path().to_path_buf();
        triple.set_output_path(Some(tmp_path.clone()));

        // We'll write two lines, each is a valid `BatchResponseRecord` in JSON:
        let line_1 = r#"{"id":"batch_req_mock_item_1","custom_id":"mock_item_1","response":{"status_code":200,"request_id":"resp_req_mock_item_1","body":{"id":"someid123","object":"chat.completion","created":0,"model":"test-model","choices":[{"index":0,"message":{"role":"assistant","content":"{\"name\":\"item-from-output-file\"}"},"logprobs":null,"finish_reason":"stop"}],"usage":{"prompt_tokens":0,"completion_tokens":0,"total_tokens":0}}}}"#;
        let line_2 = r#"{"id":"batch_req_mock_item_2","custom_id":"mock_item_2","response":{"status_code":200,"request_id":"resp_req_mock_item_2","body":{"id":"someid456","object":"chat.completion","created":0,"model":"test-model-2","choices":[{"index":0,"message":{"role":"assistant","content":"{\"name\":\"another-output-item\"}"},"logprobs":null,"finish_reason":"stop"}],"usage":{"prompt_tokens":10,"completion_tokens":10,"total_tokens":20}}}}"#;

        // Write them to the file as NDJSON (each on its own line):
        write!(tmpfile, "{}\n{}\n", line_1, line_2).unwrap();
        tmpfile.flush().unwrap();

        let result = process_output_file::<OutputFileMockItem>(
            &triple,
            workspace.as_ref(),
            &ExpectedContentType::Json,
        ).await;

        assert!(
            result.is_ok(),
            "Expected process_output_file to succeed with valid NDJSON lines"
        );
        debug!("test_process_output_file_ok passed successfully.");
    }
}
