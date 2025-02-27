// ---------------- [ File: src/check_and_download.rs ]
crate::ix!();

pub async fn check_for_and_download_output_and_error_online(
    triple: &mut BatchFileTriple,
    client: &OpenAIClientHandle,

) -> Result<(), BatchDownloadError> {

    loop {
        match check_batch_status_online(triple,client).await {
            Ok(status) => {
                info!("batch online status: {:#?}", status);

                // Download files if available
                if status.output_file_available() {
                    download_output_file(triple,client).await?;
                }
                if status.error_file_available() {
                    download_error_file(triple,client).await?;
                }
                return Ok(());
            }
            Err(BatchDownloadError::BatchStillProcessing { batch_id }) => {
                // Batch is still processing; decide whether to wait or exit
                info!("Batch {} is still processing.", batch_id);
                client.wait_for_batch_completion(&batch_id).await?;
            }
            Err(e) => {
                // Other errors
                return Err(e);
            }
        }
    }
}

pub async fn check_batch_status_online(
    triple: &BatchFileTriple,
    client: &OpenAIClientHandle,

) -> Result<BatchOnlineStatus, BatchDownloadError> {

    info!("checking batch status online");

    // Load batch metadata to get the batch ID
    let metadata_filename = triple.metadata_filename_which_maybe_does_not_yet_exist();
    let mut metadata      = BatchMetadata::load_from_file(&metadata_filename).await?;
    let batch_id          = metadata.batch_id().to_string();

    // Retrieve batch status from the API
    let batch = client.retrieve_batch(&batch_id).await?;

    match batch.status {
        BatchStatus::Completed => {

            // Update metadata with file IDs
            metadata.set_output_file_id(batch.output_file_id.clone());
            metadata.set_error_file_id(batch.error_file_id.clone());
            metadata.save_to_file(&metadata_filename).await?;

            Ok(BatchOnlineStatus::from(&batch))
        }
        BatchStatus::Failed => {
            Err(BatchDownloadError::BatchFailed {
                batch_id,
            })
        }
        BatchStatus::Validating | BatchStatus::InProgress | BatchStatus::Finalizing => {
            // Batch is still processing
            Err(BatchDownloadError::BatchStillProcessing {
                batch_id,
            })
        }
        _ => {
            // Handle other statuses if necessary
            Err(BatchDownloadError::UnknownBatchStatus {
                batch_id,
                status: batch.status.clone(),
            })
        }
    }
}

pub async fn download_output_file(
    triple: &mut BatchFileTriple,
    client: &OpenAIClientHandle,

) -> Result<(), BatchDownloadError> {

    info!("downloading batch output file");

    if triple.output().is_some() {
        return Err(BatchDownloadError::OutputFileAlreadyExists { triple: triple.clone() });
    }

    let metadata_filename = triple.metadata_filename_which_maybe_does_not_yet_exist();
    let metadata          = BatchMetadata::load_from_file(&metadata_filename).await?;
    let output_file_id    = metadata.output_file_id()?;

    // Download the output file content
    let file_content = client.file_content(output_file_id).await?;

    // Write the content to the output file
    let output_path = triple.output_filename_which_maybe_does_not_yet_exist().clone();

    assert!(!output_path.exists());

    std::fs::write(&output_path, file_content)?;

    // Update the triple with the output file path
    triple.set_output_path(Some(output_path));

    Ok(())
}

pub async fn download_error_file(
    triple: &mut BatchFileTriple,
    client: &OpenAIClientHandle,

) -> Result<(), BatchDownloadError> {

    info!("downloading batch error file");

    if triple.error().is_some() {
        return Err(BatchDownloadError::ErrorFileAlreadyExists { triple: triple.clone() });
    }

    let metadata_filename = triple.metadata_filename_which_maybe_does_not_yet_exist();
    let metadata          = BatchMetadata::load_from_file(&metadata_filename).await?;
    let error_file_id     = metadata.error_file_id()?;

    // Download the error file content
    let file_content = client.file_content(error_file_id).await?;

    // Write the content to the error file
    let error_path = triple.error_filename_which_maybe_does_not_yet_exist();

    assert!(!error_path.exists());

    std::fs::write(&error_path, file_content)?;

    // Update the triple with the error file path
    triple.set_error_path(Some(error_path));

    Ok(())
}
