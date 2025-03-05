// ---------------- [ File: get-file-size/src/count_lines_in_file.rs ]
crate::ix!();

/// Asynchronously counts the number of lines in a file
pub async fn count_lines_in_file(file_path: &PathBuf) -> Result<usize, FileError> {

    let file            = File::open(file_path).await.map_err(|e| FileError::OpenError { io: e.into() })?;
    let reader          = BufReader::new(file);
    let mut lines_count = 0;
    let mut lines       = reader.lines();

    while let Some(_) = lines.next_line().await.map_err(|e| FileError::GetNextLineError { io: e.into() })? {
        lines_count += 1;
    }

    Ok(lines_count)
}

#[cfg(test)]
mod test_count_lines_in_file {
    use std::path::{PathBuf};
    use std::io::Write as IoWrite;
    use tokio::fs::{File};
    use tokio::io::{AsyncWriteExt};
    use tempfile::NamedTempFile;
    use super::*;

    /// Verifies that counting lines in an empty file returns 0.
    #[tokio::test]
    async fn test_count_empty_file_returns_0() {
        // Create a temporary file with no content
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let file_path = temp_file.path().to_path_buf();

        let result = count_lines_in_file(&file_path).await;
        assert!(result.is_ok(), "Expected Ok result for empty file");
        assert_eq!(result.unwrap(), 0, "Empty file should have 0 lines");
    }

    /// Verifies that counting lines in a file with one line returns 1.
    #[tokio::test]
    async fn test_count_single_line_file_returns_1() {
        // Create a temporary file with one line of text
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        writeln!(temp_file, "Hello, World!").expect("Failed to write to temp file");
        let file_path = temp_file.path().to_path_buf();

        let result = count_lines_in_file(&file_path).await;
        assert!(result.is_ok(), "Expected Ok result for single-line file");
        assert_eq!(result.unwrap(), 1, "File with one line should have 1 line");
    }

    /// Verifies that counting lines in a file with multiple lines (including blank lines) is correct.
    #[tokio::test]
    async fn test_count_multiple_lines_including_blank() {
        // Create a temporary file with multiple lines of text
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        write!(temp_file,
            "Line1\n\
             Line2\n\
             \n\
             Line4\n"
        ).expect("Failed to write to temp file");
        let file_path = temp_file.path().to_path_buf();

        let result = count_lines_in_file(&file_path).await;
        assert!(result.is_ok());
        // We have 4 lines total, including a blank line for line 3
        assert_eq!(result.unwrap(), 4, "File should have 4 lines total (3 visible + 1 blank)");
    }

    /// Verifies that counting lines in a file with CRLF line endings returns the correct number of lines.
    #[tokio::test]
    async fn test_count_crlf_line_endings() {
        // Create a temporary file with CRLF endings
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let crlf_content = b"Line1\r\nLine2\r\nLine3\r\n";
        temp_file.write_all(crlf_content).expect("Failed to write CRLF content");
        let file_path = temp_file.path().to_path_buf();

        let result = count_lines_in_file(&file_path).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3, "CRLF file should have 3 lines");
    }

    /// Verifies that the function returns an error for a non-existent file.
    #[tokio::test]
    async fn test_count_lines_in_non_existent_file() {
        let non_existent = PathBuf::from("this/file/does/not/exist.txt");
        let result = count_lines_in_file(&non_existent).await;
        assert!(result.is_err(), "Expected an error for a non-existent file");
        match result {
            Err(FileError::OpenError { .. }) => { /* This is the expected variant */ }
            _ => panic!("Expected FileError::OpenError for non-existent file"),
        }
    }

    /// Verifies that the function handles unexpected I/O errors (simulated by removing file after open).
    ///
    /// Note: This test is somewhat contrived, because it's hard to provoke an actual
    ///       `GetNextLineError` consistently. We'll demonstrate forcibly removing the file
    ///       after it's opened, so subsequent reads fail. If your platform or filesystem
    ///       doesn't allow that seamlessly, consider skipping or adjusting this test.
    #[tokio::test]
    async fn test_count_lines_in_file_unexpected_io_error() {
        // Create a temporary file with a bit of content
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let file_path = temp_file.path().to_path_buf();
        {
            let mut handle = temp_file.reopen().expect("Failed to reopen for writing");
            writeln!(handle, "Should cause I/O error").expect("Failed to write content");
        }

        // We'll open the file, then forcibly remove it before reading lines, if possible.
        // This forces an I/O error while reading lines in some environments.
        let file = File::open(&file_path).await.expect("Could not open file?");
        drop(file); // close the file so we can remove it
        std::fs::remove_file(&file_path).expect("Failed to remove file for forced I/O error test");

        let result = count_lines_in_file(&file_path).await;
        assert!(result.is_err(), "Expected an error after file removal");
        match result {
            Err(FileError::OpenError { .. }) => { /* This is the expected variant */ }
            _ => panic!("Expected FileError::OpenError for forced I/O error"),
        }
    }
}
