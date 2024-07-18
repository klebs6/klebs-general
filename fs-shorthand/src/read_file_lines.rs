crate::ix!();

/// Reads all non-empty lines from a file specified by `file_path`.
///
/// # Arguments
///
/// * `file_path` - A string slice that holds the path to the file.
///
/// # Returns
///
/// This function returns an `io::Result<Vec<String>>` which is Ok containing a vector of non-empty,
/// trimmed strings if the file is successfully read, or an Err containing the I/O error if an error occurs.
pub fn read_file_lines(file_path: &str) -> io::Result<Vec<String>> {

    let file = File::open(file_path)?;

    let lines = BufReader::new(file)
        .lines()
        .collect::<Result<Vec<_>, _>>()? // Collect into a Result<Vec<String>, io::Error>, handling potential errors at this stage
        .into_iter()
        .map(|line| line.trim().to_string()) // Trim each line
        .filter(|line| !line.is_empty()) // Filter out empty lines
        .collect::<Vec<String>>();

    Ok(lines)
}
