# batch-mode-batch-executor

This crate provides functionality for executing and managing batch processing workflows in the context of OpenAI batch operations. It allows you to upload input files, create batches, monitor batch status, and download results. The crate integrates with other modules in the batch mode ecosystem, ensuring seamless execution of complex batch processes involving file uploads, status monitoring, error handling, and output reconciliation.

## Key Features
- **Batch Execution**: Handles the full lifecycle of a batch, including uploading input files, creating batches, and waiting for completion.
- **File Management**: Downloads output and error files associated with batches once processing is complete.
- **Error Handling**: Manages errors related to batch execution, file download, and reconciliation.
- **Metadata Management**: Saves and loads metadata for each batch, ensuring that batch information is consistently tracked.
- **Reconciliation**: Supports reconciliation between input and output data to ensure accurate processing.

## Usage

### Executing Batch Processing

```rust
use batch_mode_batch_executor::{fresh_execute_batch_processing, BatchFileTriple, OpenAIClientHandle};

let client = OpenAIClientHandle::new();
let mut triple = BatchFileTriple::new(input_file, output_file, error_file);
let result = triple.fresh_execute(&mut triple, &client).await?;
```

### Error Handling

This crate utilizes `error-tree` for structured error handling, supporting various error types such as `BatchProcessingError`, `BatchMetadataError`, and `BatchDownloadError`.

## Requirements
- The `OPENAI_API_KEY` environment variable must be set with your OpenAI API key.
- The crate works in conjunction with other `batch-mode` crates, such as `batch-mode-batch-client`, `batch-mode-batch-schema`, and more.

## License
This crate is licensed under the MIT License. See LICENSE for details.
