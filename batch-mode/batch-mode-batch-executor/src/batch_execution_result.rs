// ---------------- [ File: src/batch_execution_result.rs ]
crate::ix!();

#[derive(Debug,Getters)]
#[getset(get="pub")]
pub struct BatchExecutionResult {
    outputs: Option<BatchOutputData>,
    errors:  Option<BatchErrorData>,
}

impl BatchExecutionResult {

    pub fn new(outputs: Option<BatchOutputData>, errors: Option<BatchErrorData>) -> Self {
        Self { outputs, errors }
    }
}

pub async fn fresh_execute_batch_processing(triple: &mut BatchFileTriple, client: &OpenAIClientHandle) 
    -> Result<BatchExecutionResult, BatchProcessingError> 
{
    assert!(triple.input().is_some());
    assert!(triple.output().is_none());
    assert!(triple.error().is_none());
    assert!(triple.associated_metadata().is_none());

    info!("executing fresh batch processing for triple {:#?}", triple);

    let input_filename    = triple.input_filename_which_maybe_does_not_yet_exist();
    let output_filename   = triple.output_filename_which_maybe_does_not_yet_exist();
    let error_filename    = triple.error_filename_which_maybe_does_not_yet_exist();
    let metadata_filename = triple.metadata_filename_which_maybe_does_not_yet_exist();

    info!("input_filename: {:?}",    input_filename);
    info!("output_filename: {:?}",   output_filename);
    info!("error_filename: {:?}",    error_filename);
    info!("metadata_filename: {:?}", metadata_filename);

    assert!(input_filename.exists());
    assert!(!output_filename.exists());
    assert!(!error_filename.exists());
    assert!(!metadata_filename.exists());

    // Upload file
    let input_file = client.upload_batch_file(&input_filename).await?;

    let input_file_id = input_file.id;

    // Create batch
    let batch    = client.create_batch(&input_file_id).await?;
    let batch_id = batch.id.clone();

    // ** Save batch_id to metadata file **
    let mut metadata = BatchMetadata::with_input_id_and_batch_id(&input_file_id, &batch_id);
    metadata.save_to_file(&metadata_filename).await?;

    // Wait for completion
    let completed_batch = client.wait_for_batch_completion(&batch_id).await?;

    // Download output file
    let outputs = if let Some(output_file_id) = completed_batch.output_file_id {
        metadata.set_output_file_id(Some(output_file_id));
        metadata.save_to_file(&metadata_filename).await?;
        download_output_file(triple,client).await?;
        let outputs = load_output_file(&output_filename).await?;
        Some(outputs)
    } else {
        None
    };

    // Handle errors if any
    let errors = if let Some(error_file_id) = completed_batch.error_file_id {
        metadata.set_error_file_id(Some(error_file_id));
        metadata.save_to_file(&metadata_filename).await?;
        download_error_file(triple,client).await?;
        let errors = load_error_file(&error_filename).await?;
        Some(errors)
    } else {
        None
    };

    Ok(BatchExecutionResult::new(outputs,errors))
}
