crate::ix!();

/// Asynchronously counts the number of lines in a file
pub async fn count_lines_in_file(file_path: &PathBuf) -> Result<usize, WorkspaceError> {

    let file            = File::open(file_path).await.map_err(|e| FileError::OpenError { io: e.into() })?;
    let reader          = BufReader::new(file);
    let mut lines_count = 0;
    let mut lines       = reader.lines();

    while let Some(_) = lines.next_line().await.map_err(|e| FileError::GetNextLineError { io: e.into() })? {
        lines_count += 1;
    }

    Ok(lines_count)
}
