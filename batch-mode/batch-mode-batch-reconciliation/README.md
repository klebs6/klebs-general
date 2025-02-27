# batch-mode-batch-reconciliation

The `batch-mode-batch-reconciliation` crate provides functionality for reconciling batch files during a batch processing workflow. This includes determining and executing a series of reconciliation steps based on the current state of batch files (input, output, and error files). The crate supports both automated batch reconciliation and error recovery, and it is designed to integrate with other modules in the batch-mode ecosystem.

## Key Features
- **Batch Reconciliation**: Provides a structured approach to reconcile the state of batch files, including the verification and processing of input, output, and error files.
- **Dynamic Course of Action**: Automatically determines the recommended course of action based on the current state of the batch files (e.g., processing error files, ensuring matching request IDs).
- **Reconciliation Operations**: Supports various operations, such as ensuring consistency between input and output file IDs, processing batch files, and moving files to the "done" directory.
- **Error Handling and Recovery**: Handles errors such as missing files, failed operations, and mismatched request IDs, with detailed logging and retry mechanisms for batch error processing.
- **Customizable Processing**: Allows users to define custom actions for processing batch output and error files, with support for different content types (e.g., JSON, PlainText).

## Usage

### Reconcile Unprocessed Batch Triple

The main function of this crate is `reconcile_unprocessed_batch_triple`, which executes the reconciliation process for a batch file.

```rust
use batch_mode_batch_reconciliation::reconcile_unprocessed_batch_triple;
use batch_mode_batch_client::OpenAIClientHandle;
use batch_mode_batch_triple::BatchFileTriple;
use batch_mode_json::ExpectedContentType;

let client = OpenAIClientHandle::new();
let mut triple = BatchFileTriple::new(input_file, output_file, error_file);
let expected_content_type = ExpectedContentType::Json;

reconcile_unprocessed_batch_triple(&mut triple, &client, &expected_content_type, process_output_fn, process_error_fn).await?;
```

### Custom Processing Functions

Define custom functions for processing batch output and error files. These functions will be invoked during the reconciliation process.

```rust
async fn process_output_fn(triple: &BatchFileTriple, workspace: &dyn BatchWorkspaceInterface, expected_content_type: &ExpectedContentType) {
    // Custom logic for processing batch output
}

async fn process_error_fn(triple: &BatchFileTriple, operations: &[BatchErrorFileProcessingOperation]) {
    // Custom logic for processing batch error files
}
```

### Error Handling

The crate uses `error-tree` for detailed error handling. The main errors include:
- `BatchReconciliationError::MissingBatchInputFileButOthersExist`: Missing the input file but output/error files exist.
- `BatchReconciliationError::ReconciliationFailed`: The reconciliation process failed due to errors in one or more steps.

## Batch Reconciliation Operations

The crate supports several predefined reconciliation operations, such as:
- **EnsureInputRequestIdsMatchErrorRequestIds**: Ensure input file request IDs match error file request IDs.
- **ProcessBatchErrorFile**: Process the error file for the batch.
- **MoveBatchInputAndOutputToTheDoneDirectory**: Move the input and output files to the "done" directory once processed.
- **CheckForBatchOutputAndErrorFileOnline**: Check for and download output and error files from the server if they are online.

## License
This crate is licensed under the MIT License. See LICENSE for details.
