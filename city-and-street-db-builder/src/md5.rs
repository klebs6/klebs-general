crate::ix!();

/// Fetch the MD5 checksum for the given OSM PBF download link.
/// The `.md5` file contains just the MD5 sum and possibly a trailing newline.
pub async fn fetch_md5_for_region(download_link: &str) -> Result<String, PbfDownloadError> {
    let md5_url = format!("{}.md5", download_link);

    let client = reqwest::Client::new();
    let response = client.get(&md5_url)
        .send()
        .await?
        .error_for_status()?;

    let text = response.text().await?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(PbfDownloadError::IoError(
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Empty MD5 sum")
        ));
    }

    // The line is something like: "<hash>  <filename>"
    // We only want the hash, so split by whitespace and take the first part.
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.is_empty() {
        return Err(PbfDownloadError::IoError(
            std::io::Error::new(std::io::ErrorKind::InvalidData, "No hash found in MD5 file")
        ));
    }

    let md5_sum = parts[0];
    Ok(md5_sum.to_string())
}
