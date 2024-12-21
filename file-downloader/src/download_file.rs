crate::ix!();

pub async fn download_file(
    download_link: &str,
    target_file: impl AsRef<Path> + Debug,
) -> Result<(), DownloadError> {
    if let Some(parent) = target_file.as_ref().parent() {
        info!("Ensuring parent directory exists: {}", parent.display());
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(DownloadError::IoError)?;
    }

    info!("Creating file at: {}", target_file.as_ref().display());
    assert!(!target_file.as_ref().exists());
    info!("Downloading data to: {:?}", target_file);

    let client = reqwest::Client::new();
    let response = client
        .get(download_link)
        .send()
        .await?;

    info!("HTTP Response Status: {:?}", response.status());
    response.error_for_status_ref()?; // Check for non-success status codes

    let total_size = response.content_length();
    let mut file = File::create(&target_file)
        .await
        .map_err(DownloadError::IoError)?;
    let mut downloaded = 0u64;

    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.try_next().await? {
        file.write_all(&chunk)
            .await
            .map_err(DownloadError::IoError)?;
        downloaded += chunk.len() as u64;

        if let Some(total) = total_size {
            let pct = (downloaded as f64 / total as f64) * 100.0;
            info!("Download progress: {:.2}%", pct);
        } else {
            info!("Downloaded {} bytes", downloaded);
        }
    }

    info!("File downloaded successfully: {:?}", target_file);
    Ok(())
}
