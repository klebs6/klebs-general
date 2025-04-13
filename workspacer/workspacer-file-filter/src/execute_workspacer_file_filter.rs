crate::ix!();

#[tracing::instrument(level="trace", skip(writer, requests))]
pub async fn execute_ai_file_filter_requests(
    writer: &mut AiFileFilter,
    requests: &[AiFileFilterRequest<PathBuf>],
    plant: bool
) -> Result<(), AiFileFilterError> {
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
        let path_to_write = request.file_path();
        trace!("Writing filtered content to file {:?}", path_to_write.as_ref());
        let mut file = tokio::fs::File::create(path_to_write.as_ref()).await.map_err(|io_err| {
            AiFileFilterError::IoError {
                io_error: std::sync::Arc::new(io_err),
                context: format!("Failed to create or open file at {}", path_to_write.as_ref().display()),
            }
        })?;

        file.write_all(response.filtered_text().as_bytes())
            .await
            .map_err(|io_err| {
                AiFileFilterError::IoError {
                    io_error: std::sync::Arc::new(io_err),
                    context: format!("Failed to write file at {}", path_to_write.as_ref().display()),
                }
            })?;

        info!("Successfully updated file at {:?}", path_to_write.as_ref());
    }

    info!("Successfully completed execute_ai_file_filter_requests.");
    Ok(())
}
