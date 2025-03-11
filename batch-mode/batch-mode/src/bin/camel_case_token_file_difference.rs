// ---------------- [ File: src/bin/camel_case_token_file_difference.rs ]
use structopt::StructOpt;
use batch_mode::*;
use batch_mode_3p::*;
use batch_mode_3p::io::AsyncReadExt;

/// Command-line arguments for our "retain items in fileA that do not show up in fileB" program.
#[derive(StructOpt, Debug)]
pub struct FileDifferenceArgs {
    /// Path to the first file (fileA)
    #[structopt(long,parse(from_os_str))]
    file_a: PathBuf,

    /// Path to the second file (fileB)
    #[structopt(long,parse(from_os_str))]
    file_b: PathBuf,

    /// Path to the output file (fileC)
    #[structopt(long,parse(from_os_str))]
    file_c: PathBuf,
}

impl FileDifferenceArgs {

    /// Async function which computes the difference of tokens in fileA and fileB (ignoring comments)
    /// and writes the result to fileC.
    pub async fn run(
        &self,
    ) -> Result<(), TokenParseError> {

        // Parse the lines in fileA and fileB as CamelCaseTokenWithComment
        let tokens_a = parse_token_file(self.file_a.to_str().unwrap()).await?;
        let tokens_b = parse_token_file(self.file_b.to_str().unwrap()).await?;

        info!("tokens_a, len={}", tokens_a.len());
        info!("tokens_b, len={}", tokens_b.len());

        // We'll gather the `data` fields from fileB into a set for quick membership checks.
        let data_in_b: HashSet<String> = tokens_b
            .into_iter()
            .map(|token| token.name().to_string())
            .collect();

        info!("data_in_b, len={}", data_in_b.len());

        // Filter out those tokens from fileA whose `data` fields appear in fileB.
        let filtered_tokens: Vec<CamelCaseTokenWithComment> = tokens_a
            .into_iter()
            .filter(|token| !data_in_b.contains(&token.name().to_string()))
            .collect();

        info!("filtered_tokens, len={}", filtered_tokens.len());

        // Write the remaining tokens to fileC. We reuse Display impl of CamelCaseTokenWithComment,
        // which produces "Data -- comment" if comment exists, or just "Data" otherwise.
        let mut out_file = File::create(&self.file_c).await?;
        for token in filtered_tokens {
            let line = format!("{}\n", token);
            out_file.write_all(line.as_bytes()).await?;
        }

        Ok(())
    }
}

/// Entry point for the binary. Parses CLI arguments, then calls our async difference function.
#[tokio::main]
pub async fn main() -> Result<(), TokenParseError> {
    configure_tracing();
    let args = FileDifferenceArgs::from_args();
    args.run().await?;
    Ok(())
}

/// Example tests to show how you might confirm the difference logic works. Adjust as needed.
#[cfg(test)]
mod test_file_difference_logic {
    use super::*;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_difference_of_files() -> Result<(), TokenParseError> {
        // Prepare some in-memory files using tempfile (for illustration).
        let dir = tempfile::tempdir()?;
        let path_a = dir.path().join("fileA.txt");
        let path_b = dir.path().join("fileB.txt");
        let path_c = dir.path().join("fileC.txt");

        // Write sample data to fileA
        {
            let mut file_a = File::create(&path_a).await?;
            file_a.write_all(b"TokenA -- comment A\nTokenB -- comment B\nTokenC -- comment C\n").await?;
        }

        // Write sample data to fileB
        {
            let mut file_b = File::create(&path_b).await?;
            file_b.write_all(b"TokenA -- different comment\nTokenC -- another comment\n").await?;
        }

        // Run the difference
        let cli_args = FileDifferenceArgs {
            file_a: path_a.clone(),
            file_b: path_b.clone(),
            file_c: path_c.clone(),
        };
        cli_args.run().await?;

        // Read the contents of fileC and ensure the filtered results are correct
        let file_c = File::open(&path_c).await?;
        let mut reader_c = BufReader::new(file_c);
        let mut output = String::new();
        reader_c.read_to_string(&mut output).await?;

        // We expect only TokenB to remain
        assert!(output.contains("TokenB -- comment B"));
        assert!(!output.contains("TokenA"));
        assert!(!output.contains("TokenC"));

        Ok(())
    }
}
