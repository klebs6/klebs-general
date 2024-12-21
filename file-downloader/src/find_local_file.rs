crate::ix!();

pub fn find_local_file(
    filename:  impl AsRef<Path>, 
    directory: impl AsRef<Path>,
    extension: &str,

) -> Result<Option<String>, DownloadError> {

    let base_name = base_filename_without_extension(&filename, extension);

    let dir_iter = match std::fs::read_dir(directory) {
        Ok(iter) => iter,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(DownloadError::IoError(e)),
    };

    for entry in dir_iter {
        let entry = entry.map_err(DownloadError::IoError)?;
        let fname = entry.file_name().to_string_lossy().to_string();
        if fname.starts_with(&base_name) && fname.ends_with(extension) {
            return Ok(Some(fname));
        }
    }

    Ok(None)
}
