// ---------------- [ File: src/process_output_file.rs ]
crate::ix!();

/**
 * The core async function to process the output file 
 * for a given triple. Now requiring T: 'static + Send + Sync.
 */
pub async fn process_output_file<T>(
    triple:                &BatchFileTriple,
    workspace:             &dyn BatchWorkspaceInterface,
    expected_content_type: &ExpectedContentType,
) -> Result<(), BatchOutputProcessingError> 
where
    T: 'static + Send + Sync + DeserializeOwned + Named + GetTargetPathForAIExpansion,
{
    trace!("process_output_file => index = {:?}", triple.index());
    let output_data = load_output_file(triple.output().as_ref().unwrap()).await?;
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

/// A non-generic fallback bridging function:
pub fn default_output_file_bridge_fn<'a>(
    triple:    &'a BatchFileTriple,
    workspace: &'a (dyn BatchWorkspaceInterface + 'a),
    ect:       &'a ExpectedContentType,
) -> Pin<Box<dyn Future<Output = Result<(), BatchOutputProcessingError>> + Send + 'a>>
{
    Box::pin(async move {
        // Use CamelCaseTokenWithComment (or another default type) here:
        process_output_file::<CamelCaseTokenWithComment>(triple, workspace, ect).await
    })
}

/// The const pointer the macro references.
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
        let workspace = Arc::new(MockWorkspace::default());
        let mut triple = BatchFileTriple::new_for_test_empty();
        triple.set_input_path(Some("dummy_input.json".into()));
        triple.set_output_path(Some("dummy_output.json".into()));

        let msg = BatchMessageBuilder::default()
            .role(MessageRole::Assistant)
            .content(
                BatchMessageContentBuilder::default()
                    .content(r#"{"name":"item-from-output-file"}"#.to_string())
                    .build()
                    .unwrap()
            )
            .build()
            .unwrap();

        let choice = BatchChoiceBuilder::default()
            .index(0_u32)
            .finish_reason(FinishReason::Stop)
            .logprobs(None)
            .message(msg)
            .build()
            .unwrap();

        let success_body = BatchSuccessResponseBodyBuilder::default()
            .id("someid123".to_string())
            .object("response".to_string())
            .created(0_u64)
            .model("test-model".to_string())
            .choices(vec![choice])
            .usage(BatchUsage::mock())
            .build()
            .unwrap();

        let success_content = BatchResponseContentBuilder::default()
            .status_code(200_u16)
            .request_id(ResponseRequestId::new("resp_req_mock_item_1"))
            .body(BatchResponseBody::Success(success_body))
            .build()
            .unwrap();

        // FIX: Supply id(...) so the builder doesn't fail with "UninitializedField('id')"
        let record_ok = BatchResponseRecordBuilder::default()
            .id(BatchRequestId::new("batch_req_mock_item_1"))
            .custom_id(CustomRequestId::new("mock_item_1"))
            .response(success_content)
            .build()
            .unwrap();

        let output_data = BatchOutputData::new(vec![record_ok]);
        let text = serde_json::to_string_pretty(&output_data).unwrap();
        std::fs::write("dummy_output.json", text).unwrap();

        let result = process_output_file::<OutputFileMockItem>(
            &triple,
            workspace.as_ref(),
            &ExpectedContentType::Json
        ).await;
        assert!(result.is_ok());
    }
}
