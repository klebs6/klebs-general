crate::ix!();

pub async fn download_file_with_md5(
    download_link:      &str,
    filename:           impl AsRef<Path>, 
    directory:          impl AsRef<Path>, 
    maybe_expected_md5: Option<&str>
) -> Result<PathBuf, DownloadError> {

    let temp_file = directory.as_ref().join(&filename);
    if temp_file.exists() {
        std::fs::remove_file(&temp_file).map_err(DownloadError::IoError)?;
    }

    download_file(download_link, &temp_file).await?;

    match maybe_expected_md5 {
        Some(expected_md5) => {
            verify_md5_checksum(&temp_file, expected_md5).await?;

            let final_name = filename_with_md5(&filename, expected_md5);
            let final_path = directory.as_ref().join(&final_name);

            if final_path.exists() {
                std::fs::remove_file(&final_path).map_err(DownloadError::IoError)?;
            }

            // Only rename if the destination is different from the temp file
            if final_path != temp_file {
                std::fs::rename(&temp_file, &final_path)
                    .map_err(DownloadError::IoError)?;
                Ok(final_path)
            } else {
                Ok(temp_file)
            }
        },
        None => {
            // No MD5 to check, so just return the temp file path
            Ok(temp_file)
        }
    }
}
