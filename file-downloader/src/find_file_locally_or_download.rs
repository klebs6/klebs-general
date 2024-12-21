crate::ix!();

/// Attempt to obtain a file locally (with MD5 check) or download it otherwise.
pub async fn find_file_locally_or_download(
    download_link:     &str,
    md5_download_link: Option<&str>,
    directory:         impl AsRef<Path>,
) -> Result<PathBuf, DownloadError> {

    let filename  = filename(download_link);
    let extension = get_extension_intelligent(&filename).expect("expect our file to have an extension");

    // 1) Check if a suitable local file exists
    if let Some(local_path) = try_use_local_file(&filename, &directory, extension.as_str()).await? {
        return Ok(local_path);
    }

    // 2) Fetch the expected MD5
    let maybe_expected_md5 = if let Some(link) = md5_download_link {
        Some(fetch_md5_for_link(&link).await?)
    } else {
        None
    };

    // 3) Download the file, renaming it to include the MD5
    let fresh_path = download_file_with_md5(
        download_link,
        &filename,
        &directory,
        maybe_expected_md5.as_deref(),
    )
    .await?;

    Ok(fresh_path)
}

/// Look for a local file with the given base name and extension, checking for a valid MD5.
/// If it’s valid, return `Some(local_path)`. Otherwise, return `None`.
pub async fn try_use_local_file(
    base_filename: &Path,
    directory:     impl AsRef<Path>,
    extension:     &str,

) -> Result<Option<PathBuf>, DownloadError> {

    // Attempt to find a local file that matches the expected pattern
    if let Some(local_file) = find_local_file(base_filename, &directory, extension)? {
        let local_file_path = directory.as_ref().join(&local_file);

        // Extract the embedded MD5 from the file name
        if let Some(filename_md5) = extract_md5_from_filename(&local_file) {

            // Compute MD5 for the file we found
            let actual_md5 = compute_md5(&local_file_path).await?;

            if actual_md5 == filename_md5 {
                info!("Local file checksum matches filename: {:?}", local_file_path);
                return Ok(Some(local_file_path));
            } else {
                warn!(
                    "Local file checksum mismatch! Expected: {}, Actual: {}",
                    filename_md5, actual_md5
                );
                warn!("Will download a fresh copy...");
            }
        } else {
            info!("Local file lacks MD5, will download a fresh copy: {:?}", local_file_path);
        }
    }

    Ok(None)
}
