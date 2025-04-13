// ---------------- [ File: workspacer-file-filter/src/execute_workspacer_file_filter.rs ]
crate::ix!();

pub async fn execute_ai_file_filter_requests(
    writer:   &mut AiFileFilter,
    requests: &[AiFileFilterRequest<PathBuf>],
    plant:    bool
) -> Result<(), AiFileFilterError>
{
    let unseen = writer
        .batch_workspace()
        .calculate_unseen_inputs(&requests, &ExpectedContentType::Json);

    info!("Gathering AI expansions for file filtering. unseen inputs={:#?}", unseen);

    if plant {
        writer.plant_seed_and_wait(&unseen).await?;
    }

    let results = writer.gather_results(&unseen).await?;

    debug!("Gathered results={:#?}", results);

    // For each (request, response), write `filtered_text` back to the original file
    for (request, response) in results {

        //remember, these are the types
        let request: AiFileFilterRequest<PathBuf> = request;
        let response: AiFileFilterDesiredOutput = response;

        let path_to_write = request.file_path();

        trace!("Writing filtered content to file {:?}", path_to_write);

        // Fix type annotation need by binding a local reference
        let mut file = tokio::fs::File::create(&path_to_write).await.map_err(|io_err| {
            AiFileFilterError::IoError {
                io_error: std::sync::Arc::new(io_err),
                context: format!("Failed to create or open file at {}", path_to_write.display()),
            }
        })?;

        file.write_all(response.filtered_text().as_bytes()).await.map_err(|io_err| {
            AiFileFilterError::IoError {
                io_error: std::sync::Arc::new(io_err),
                context: format!("Failed to write file at {}", path_to_write.display()),
            }
        })?;

        info!("Successfully updated file at {:?}", path_to_write);
    }

    info!("Successfully completed execute_ai_file_filter_requests.");
    Ok(())
}
