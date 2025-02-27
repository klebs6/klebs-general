# batch-mode-batch-triple

The `batch-mode-batch-triple` crate provides the `BatchFileTriple` structure and associated methods for managing and processing batch files within a batch processing system. It integrates with the rest of the batch-mode ecosystem to handle batch file states, perform error handling, validate file consistency, and move batch files to a "done" directory upon completion.

## Key Features
- **BatchFileTriple Management**: Represents a set of batch files (input, output, error, and metadata) associated with a specific batch index.
- **File Validation**: Ensures that batch files, such as input, output, and error files, match in terms of request IDs, and validates their integrity.
- **Error Handling**: Provides methods for logging and retrying failed batch file processing requests.
- **File Movement**: Supports moving batch files to a "done" directory once processed to ensure proper workflow management.
- **State Management**: Determines the current state of batch files (input-only, input-output, input-error, or input-output-error).

## Usage

### Creating a BatchFileTriple

You can create a new `BatchFileTriple` with a set of requests and specify a workspace for file management.

```rust
use batch_mode_batch_triple::BatchFileTriple;
use batch_mode_batch_workspace_interface::BatchWorkspaceInterface;
use std::sync::Arc;

let workspace: Arc<dyn BatchWorkspaceInterface> = // obtain workspace
let batch_triple = BatchFileTriple::new_with_requests(&requests, workspace)?;
```

### File Validation

Use the `ensure_input_matches_output_and_error` method to validate that input, output, and error files have matching request IDs.

```rust
batch_triple.ensure_input_matches_output_and_error().await?;
```

### Error Handling

Log errors in the batch processing workflow with the `log_errors` method.

```rust
batch_triple.log_errors(&error_data).await?;
```

Retry failed requests using the `retry_failed_requests` method:

```rust
batch_triple.retry_failed_requests(&error_data).await?;
```

### Moving Files

After processing a batch, you can move the batch files to a "done" directory.

```rust
batch_triple.move_input_and_output_to_done().await?;
```

### Ensuring File Consistency

Ensure that the input file matches the output file:

```rust
batch_triple.ensure_input_matches_output().await?;
```

Ensure that the input file matches the error file:

```rust
batch_triple.ensure_input_matches_error().await?;
```

## Error Handling

The crate uses custom error types for managing batch file processing errors:

- `BatchErrorProcessingError`: Errors encountered while processing batch error files.
- `BatchValidationError`: Errors related to validation, such as mismatched request IDs.
- `FileMoveError`: Errors encountered when moving files to the "done" directory.

## License
This crate is licensed under the MIT License. See LICENSE for details.
