# batch-mode-batch-client

This crate provides a client for interacting with OpenAI's batch processing API, allowing you to manage and download batch files asynchronously. It offers functionality for managing batch statuses, uploading files, and retrieving results after batch processing.

## Features
- **Create and manage batches**: Create new batches for processing, retrieve batch details, and monitor batch status.
- **Upload and retrieve files**: Upload files for batch processing and retrieve file content after batch completion.
- **Handle batch statuses**: Automatically wait for batch completion and handle error cases such as batch failures or incomplete batches.
- **Download output and error files**: After batch completion, download the resulting output and error files.

## Usage

### Creating a New Client

```rust
use openai_batch_client::OpenAIClientHandle;

let client = OpenAIClientHandle::new();
```

### Creating a Batch

```rust
let batch = client.create_batch("input_file_id").await?;
```

### Uploading a File

```rust
let file = client.upload_batch_file("path_to_file").await?;
```

### Retrieving Batch Status

```rust
let batch = client.retrieve_batch("batch_id").await?;
```

### Downloading Files

```rust
use openai_batch_client::check_and_download::check_for_and_download_output_and_error_online;

check_for_and_download_output_and_error_online(&mut batch_file_triple, &client).await?;
```

### Error Handling

This crate uses `error-tree` for error management, providing structured errors such as `BatchFailed`, `BatchStillProcessing`, and other batch-related errors.

## Requirements
- `OPENAI_API_KEY` environment variable must be set with your OpenAI API key.

## License
This crate is licensed under the MIT License. See LICENSE for details.
