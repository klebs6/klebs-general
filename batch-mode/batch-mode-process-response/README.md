# batch-mode-process-response

The `batch-mode-process-response` crate is designed to manage the processing of batch responses within a larger batch-processing system. It provides functionality for handling successful responses, error data, file processing, and failed JSON repairs. This crate is an integral part of batch workflows that involve processing outputs, handling errors, and managing responses in an asynchronous and robust manner.

## Key Features
- **Process Output Data**: Process batch output data, handle successful responses, and write the data to appropriate files.
- **Error Handling**: Manage error files by logging errors and retrying failed requests.
- **Failed JSON Repair**: Log failed JSON repairs into a dedicated directory and attempt retries for broken JSON.
- **Batch Processing**: Handle processing of output and error files, including managing response content based on expected types (e.g., JSON or plain text).

## Modules and Functions

### Processing Output Data
The crate offers the `process_output_data` function to process batch output responses, handling successful responses and saving the failed ones.

```rust
pub async fn process_output_data(
    output_data: &BatchOutputData,
    workspace: &dyn BatchWorkspaceInterface,
    expected_content_type: &ExpectedContentType,
) -> Result<(), BatchOutputProcessingError>;
```

### Handle Successful Responses
It processes successful responses, including handling different content types (e.g., JSON and plain text), with options for saving them to the disk.

```rust
pub async fn handle_successful_response(
    success_body: &BatchSuccessResponseBody,
    workspace: &dyn BatchWorkspaceInterface,
    expected_content_type: &ExpectedContentType,
) -> Result<(), BatchSuccessResponseHandlingError>;
```

### Handling Failed JSON Repairs
If JSON parsing fails, the crate allows logging the failed content into a specific directory.

```rust
pub async fn handle_failed_json_repair(
    failed_id: &str,
    message_content: &BatchMessageContent,
    workspace: &dyn BatchWorkspaceInterface,
) -> Result<(), BatchSuccessResponseHandlingError>;
```

### Error File Processing
The crate also provides functionality for processing error files, including operations like logging errors and retrying failed requests.

```rust
pub async fn process_error_file(
    triple: &BatchFileTriple,
    operations: &[BatchErrorFileProcessingOperation],
) -> Result<(), BatchErrorProcessingError>;
```

## Usage

### Process Output Data
To process the batch output data, use the `process_output_data` function:

```rust
process_output_data(&output_data, workspace, &expected_content_type).await?;
```

### Handle Success and Errors
The `handle_successful_response` function is used to handle successful responses and the corresponding content type:

```rust
handle_successful_response(&success_body, workspace, &expected_content_type).await?;
```

### Failed JSON Repair Handling
If there's a failed JSON repair, use the `handle_failed_json_repair` function:

```rust
handle_failed_json_repair(&failed_id, &message_content, workspace).await?;
```

## Error Handling
The crate includes error handling for:
- **Batch Success Handling**: For managing successful response processing.
- **Batch Error Handling**: For handling errors in batch responses and retries.
- **JSON Repair Failures**: For logging and retrying failed JSON repairs.

## License
This crate is licensed under the MIT License. See LICENSE for details.
