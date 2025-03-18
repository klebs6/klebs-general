crate::ix!();

pub async fn parse_token_file(
    filename: &str
) -> Result<Vec<CamelCaseTokenWithComment>, TokenParseError>
{
    info!("Parsing token file: {}", filename);

    let file = tokio::fs::File::open(filename).await.map_err(|e| {
        error!("Failed to open file: {}. Error: {:?}", filename, e);
        TokenParseError::IoError(e)
    })?;

    let reader = tokio::io::BufReader::new(file);
    let mut lines = reader.lines();

    let mut tokens = Vec::new();

    while let Some(line) = lines.next_line().await.map_err(|e| {
        error!("Failed to read line from file: {}. Error: {:?}", filename, e);
        TokenParseError::IoError(e)
    })? {
        let trimmed = line.trim().to_string();
        if trimmed.is_empty() {
            debug!("Skipping empty line in file: {}", filename);
            continue;
        }

        match trimmed.parse::<CamelCaseTokenWithComment>() {
            Ok(token) => {
                trace!("Successfully parsed token: {}", token);
                tokens.push(token);
            }
            Err(e) => {
                warn!("Token parse error: {:?} in file: {}", e, filename);
            }
        }
    }

    info!(
        "Successfully parsed {} tokens from file: {}",
        tokens.len(),
        filename
    );
    Ok(tokens)
}

#[cfg(test)]
mod test_parse_token_file {
    use super::*;

    #[traced_test]
    async fn test_parse_token_file_valid() {
        tracing::info!("Testing parse_token_file with valid content");
        let mut tmpfile = NamedTempFile::new().unwrap();
        writeln!(tmpfile, "Token1 -- Comment1").unwrap();
        writeln!(tmpfile, "Token2").unwrap();

        let path = tmpfile.into_temp_path();
        let filename = path.to_str().unwrap();
        let tokens = parse_token_file(filename).await.unwrap();

        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].data(), "Token1");
        assert_eq!(*tokens[0].comment(), Some("Comment1".to_string()));

        assert_eq!(tokens[1].data(), "Token2");
        assert_eq!(*tokens[1].comment(), None);
    }

    #[traced_test]
    async fn test_parse_token_file_empty_lines() {
        tracing::info!("Testing parse_token_file with empty lines");
        let mut tmpfile = NamedTempFile::new().unwrap();
        writeln!(tmpfile, "").unwrap();
        writeln!(tmpfile, "   ").unwrap();
        writeln!(tmpfile, "TokenWithData").unwrap();

        let path = tmpfile.into_temp_path();
        let filename = path.to_str().unwrap();
        let tokens = parse_token_file(filename).await.unwrap();

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].data(), "TokenWithData");
        assert_eq!(*tokens[0].comment(), None);
    }

    #[traced_test]
    async fn test_parse_token_file_io_error() {
        tracing::info!("Testing parse_token_file with non-existent file");
        let result = parse_token_file("non_existent_file.txt").await;
        assert!(matches!(result, Err(TokenParseError::IoError(_))));
    }

    #[traced_test]
    async fn test_parse_token_file_invalid_lines() {
        tracing::info!("Testing parse_token_file with invalid lines");
        let mut tmpfile = NamedTempFile::new().unwrap();
        writeln!(tmpfile, "-- starts with comment").unwrap();
        writeln!(tmpfile, " ").unwrap();
        writeln!(tmpfile, "ValidToken -- ValidComment").unwrap();

        let path = tmpfile.into_temp_path();
        let filename = path.to_str().unwrap();
        let tokens = parse_token_file(filename).await.unwrap();

        // Invalid lines are ignored with warnings, only valid lines remain
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].data(), "ValidToken");
        assert_eq!(*tokens[0].comment(), Some("ValidComment".to_string()));
    }
}
