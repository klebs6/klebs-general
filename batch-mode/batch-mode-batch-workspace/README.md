# batch-mode-batch-workspace

The `batch-mode-batch-workspace` crate provides functionality for managing batch processing workspaces. It integrates with batch files and batch indices, handling tasks such as file validation, locating batch files, and gathering batch file triples. The crate also provides methods for interacting with files in the workspace, ensuring the correct structure for batch processing.

## Key Features
- **Workspace Management**: Defines a batch workspace with directories for work, logs, and completed files.
- **Batch File Location**: Locates batch files (input, output, error, and metadata) based on a batch index in the workspace.
- **Batch File Validation**: Ensures that batch files have matching request IDs across input, output, and error files.
- **File Handling**: Supports moving files to a "done" directory and other workspace-related operations.
- **Batch Index Management**: Finds and works with batch indices, including UUID-based and usize-based indices.

## Usage

### Creating a BatchWorkspace

You can create a new workspace using one of the following methods:

#### Create a Temporary Workspace

```rust
use batch_mode_batch_workspace::BatchWorkspace;
let workspace = BatchWorkspace::new_temp().await?;
```

#### Create a Workspace in a Specific Directory

```rust
let workspace = BatchWorkspace::new_in("/path/to/workspace").await?;
```

### Locating Batch Files

To locate batch files for a specific batch index:

```rust
let batch_triple = workspace.locate_batch_files(&BatchIndex::Usize(4)).await?;
```

### Gathering All Batch Files

You can gather all batch triples in the workspace with:

```rust
let batch_files = workspace.gather_all_batch_triples().await?;
```

### Handling Batch File Validation

To ensure that input files match the corresponding output and error files:

```rust
batch_triple.ensure_input_matches_output().await?;
batch_triple.ensure_input_matches_error().await?;
batch_triple.ensure_input_matches_output_and_error().await?;
```

### Error Handling

The crate defines custom error types for managing workspace and batch file errors:

- `BatchWorkspaceError`: Errors encountered during batch workspace operations, such as file access or batch index issues.
- `BatchValidationError`: Errors when validating batch files, such as mismatched request IDs.
- `FileMoveError`: Errors encountered when moving files in the workspace.

## License
This crate is licensed under the MIT License. See LICENSE for details.
