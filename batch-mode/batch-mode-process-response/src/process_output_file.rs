// ---------------- [ File: src/process_output_file.rs ]
crate::ix!();

pub async fn process_output_file(
    triple:                &BatchFileTriple,
    workspace:             &dyn BatchWorkspaceInterface,
    expected_content_type: &ExpectedContentType,
) -> Result<(), BatchOutputProcessingError> {
    let output_data = load_output_file(triple.output().as_ref().unwrap()).await?;
    process_output_data(&output_data,workspace,expected_content_type).await
}
