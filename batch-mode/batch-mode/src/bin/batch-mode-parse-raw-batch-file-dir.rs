// ---------------- [ File: src/bin/batch-mode-parse-raw-batch-file-dir.rs ]
use structopt::StructOpt;
use batch_mode::*;
use batch_mode_3p::*;

error_tree! {
    pub enum BatchModeCliError {
        PathIsNotDirectory,
        BatchWorkspaceError(BatchWorkspaceError),
        JsonParseError(JsonParseError),
        SerdeJsonError(serde_json::Error),
        Io(std::io::Error),
    }
}

/// Suffixes used by our script to locate JSON Lines files with batch data.
const OUTPUT_FILE_SUFFIX: &str = "_output.jsonl";
const ERROR_FILE_SUFFIX:  &str = "_error.jsonl";

#[derive(StructOpt, Debug)]
#[structopt(name = "batch-mode-parse-raw-batch-file-dir")]
pub struct BatchModeParseRawBatchFileDirCli {
    /// Path to the directory that contains your output/error batch files
    #[structopt(long)]
    path: Option<PathBuf>,

    /// Optional path to a separate directory containing your *original request*
    /// input files. If not provided, we won't search for input files.
    #[structopt(long)]
    input_dir: Option<PathBuf>,

    /// Optional prefix used to identify the input files in `input_dir`.
    /// For example, if your files are named "requestBatchA_001.jsonl",
    /// you might set `--input-prefix requestBatchA_`.
    /// We'll only load files that are in `--input-dir` and whose names
    /// *start with* this prefix and *end with* ".jsonl".
    #[structopt(long)]
    input_prefix: Option<String>,
}

impl BatchModeParseRawBatchFileDirCli {
    /// The main entry point for the CLI.
    ///
    /// 1) Scans the specified `--path` (or current dir) for `_output.jsonl` and `_error.jsonl` files.
    /// 2) Aggregates them into `BatchOutputData` and `BatchErrorData`.
    /// 3) (Optionally) if `--input-dir` is provided, scans that directory for files whose
    ///    filenames start with `--input-prefix` (if any) and end in `.jsonl`. We treat these
    ///    as containing the *original request lines*. We'll parse out `custom_id` and store
    ///    the entire raw line in memory for re-batching.
    /// 4) Prints the content from successful batch entries.
    /// 5) Prints the original request lines for each error record that can be matched by `custom_id`.
    pub async fn run(&self) -> Result<(), BatchModeCliError> {
        // If no directory is specified for the batch files, use the current directory.
        let mut path = PathBuf::from(".");
        if let Some(cli_path) = &self.path {
            path = cli_path.clone();
        }

        if !path.is_dir() {
            return Err(BatchModeCliError::PathIsNotDirectory);
        }

        // Gather the output & error files from `path`.
        let (error_file_paths, output_file_paths) = gather_error_and_output_files(&path)?;

        // Load and aggregate Error data
        let mut error_data_vec = Vec::new();
        for path in error_file_paths {
            error_data_vec.push(load_error_file(path).await?);
        }
        let error_data = BatchErrorData::from(error_data_vec);

        // Load and aggregate Output data
        let mut output_data_vec = Vec::new();
        for path in output_file_paths {
            output_data_vec.push(load_output_file(path).await?);
        }
        let output_data = BatchOutputData::from(output_data_vec);

        // If we have a separate directory for input, gather those original lines.
        // Otherwise, we won't attempt to re-batch from separate inputs.
        let mut all_input_lines = Vec::new();
        if let Some(input_dir) = &self.input_dir {
            if !input_dir.is_dir() {
                return Err(BatchModeCliError::PathIsNotDirectory);
            }
            let prefix = self.input_prefix.as_deref().unwrap_or("");
            let input_file_paths = gather_input_files(input_dir, prefix)?;
            for path in input_file_paths {
                let mut lines = load_raw_lines_with_custom_id(path).await?;
                all_input_lines.append(&mut lines);
            }
        }

        // 1) Print all successful batch entries (like the old Raku parse-batch-output).
        print_choice_contents(&output_data)?;

        // 2) Print a marker to indicate the lines to re-batch.
        println!("these were errors and should be re-batched:");

        // 3) For each error record, find the original line by matching custom_id, then print it.
        //    If no input lines were loaded, we won't find matches (unless your original lines
        //    happen to exist in the output data itself, which is less common).
        rebatch_errors_by_printing_original_lines(&error_data, &all_input_lines)?;

        Ok(())
    }
}

/// Searches `directory` for files that are `_output.jsonl` or `_error.jsonl`.
fn gather_error_and_output_files(
    directory: &PathBuf
) -> Result<(Vec<PathBuf>, Vec<PathBuf>), BatchModeCliError> {
    let mut error_files  = Vec::new();
    let mut output_files = Vec::new();

    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.ends_with(ERROR_FILE_SUFFIX) {
                    error_files.push(path);
                } else if name.ends_with(OUTPUT_FILE_SUFFIX) {
                    output_files.push(path);
                }
            }
        }
    }

    Ok((error_files, output_files))
}

/// Searches `directory` for files whose name starts with `prefix` and ends with `.jsonl`.
/// This can be used to find your *original request* files.
fn gather_input_files(
    directory: &PathBuf,
    prefix: &str
) -> Result<Vec<PathBuf>, BatchModeCliError> {
    let mut input_files = Vec::new();
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with(prefix) && name.ends_with(".jsonl") {
                    input_files.push(path);
                }
            }
        }
    }
    Ok(input_files)
}

/// Loads a `_error.jsonl` file into `BatchErrorData`.
pub async fn load_error_file(path: PathBuf) -> Result<BatchErrorData, BatchModeCliError> {
    let file   = File::open(&path).await.map_err(BatchModeCliError::Io)?;
    let reader = BufReader::new(file);

    let mut lines     = reader.lines();
    let mut responses = Vec::new();

    while let Some(line) = lines.next_line().await.map_err(BatchModeCliError::Io)? {
        let record: BatchResponseRecord =
            serde_json::from_str(&line).map_err(BatchModeCliError::SerdeJsonError)?;
        responses.push(record);
    }
    Ok(BatchErrorData::new(responses))
}

/// Loads a `_output.jsonl` file into `BatchOutputData`.
pub async fn load_output_file(path: PathBuf) -> Result<BatchOutputData, BatchModeCliError> {
    let file   = File::open(&path).await.map_err(BatchModeCliError::Io)?;
    let reader = BufReader::new(file);

    let mut lines     = reader.lines();
    let mut responses = Vec::new();

    while let Some(line) = lines.next_line().await.map_err(BatchModeCliError::Io)? {
        let record: BatchResponseRecord =
            serde_json::from_str(&line).map_err(BatchModeCliError::SerdeJsonError)?;
        responses.push(record);
    }
    Ok(BatchOutputData::new(responses))
}

/// Loads raw input lines that have a "custom_id" field.
/// Returns a vector of `(CustomRequestId, full_line_as_string)`.
/// This way, we can re-print the *exact* request line if we need to resubmit it.
pub async fn load_raw_lines_with_custom_id(
    path: PathBuf
) -> Result<Vec<(CustomRequestId, String)>, BatchModeCliError> {
    let file   = File::open(&path).await.map_err(BatchModeCliError::Io)?;
    let reader = BufReader::new(file);

    let mut lines = reader.lines();
    let mut output = Vec::new();

    while let Some(line) = lines.next_line().await.map_err(BatchModeCliError::Io)? {
        let value: serde_json::Value =
            serde_json::from_str(&line).map_err(BatchModeCliError::SerdeJsonError)?;
        if let Some(cid_val) = value.get("custom_id") {
            if let Some(cid_str) = cid_val.as_str() {
                let custom_id = CustomRequestId::new(cid_str.to_owned());
                output.push((custom_id, line));
            }
        }
    }
    Ok(output)
}

/// Prints the cleaned content from each choice in any successful response.
/// This is the equivalent of the old Raku `parse-batch-output` that removed
/// backtick fences before printing.
fn print_choice_contents(output_data: &BatchOutputData) -> Result<(), BatchModeCliError> {
    for record in output_data {
        let response_content = record.response();
        if response_content.is_success() {
            if let Some(success_body) = response_content.success_body() {
                for choice in success_body.choices() {
                    let raw_content = choice.message().content().as_ref();
                    let cleaned = strip_triple_backtick_fences(raw_content);
                    println!("{}", cleaned);
                }
            }
        }
    }
    Ok(())
}

/// For each error record, find the original line by matching the `custom_id`,
/// then print that raw request line so it can be re-submitted in a new batch.
fn rebatch_errors_by_printing_original_lines(
    error_data: &BatchErrorData,
    all_input_lines: &[(CustomRequestId, String)],
) -> Result<(), BatchModeCliError> {
    for error_record in error_data {
        let cid = error_record.custom_id();
        let maybe_original_line = all_input_lines
            .iter()
            .find(|(request_cid, _raw_line)| request_cid == cid);

        match maybe_original_line {
            Some((_, raw_line)) => {
                // Print the entire JSON request line
                println!("{}", raw_line);
            }
            None => {
                eprintln!(
                    "Could not find custom_id = {} in the loaded input data. \
                     This indicates the record might come from another file or directory.",
                    cid
                );
            }
        }
    }
    Ok(())
}

/// Strips a leading triple-backtick fence (optionally labeled “output”)
/// and a trailing triple-backtick from the given content,
/// as in the original Raku script.
fn strip_triple_backtick_fences(content: &str) -> String {
    let trimmed = content.trim_start();
    let trimmed = if trimmed.starts_with("```output") {
        trimmed.trim_start_matches("```output").trim_start()
    } else if trimmed.starts_with("```") {
        trimmed.trim_start_matches("```").trim_start()
    } else {
        trimmed
    };
    let trimmed = trimmed.trim_end();
    let trimmed = if trimmed.ends_with("```") {
        trimmed.trim_end_matches("```").trim_end()
    } else {
        trimmed
    };
    trimmed.to_string()
}

#[tokio::main]
async fn main() -> Result<(), BatchModeCliError> {
    configure_tracing();
    let cli = BatchModeParseRawBatchFileDirCli::from_args();
    cli.run().await
}
