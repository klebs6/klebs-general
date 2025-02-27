# batch-mode-batch-workspace-interface

The `batch-mode-batch-workspace-interface` crate defines the set of traits that outline the operations for interacting with batch workspaces in a batch processing system. These traits cover operations related to file paths, workspace directories, token expansions, and file storage paths.

## Key Features
- **Trait-based Interface**: Defines traits for interacting with batch files and directories, providing a flexible and extendable interface.
- **File Path Operations**: Supports getting paths for input, output, error, and metadata files for a specific batch index.
- **Workspace Directory Management**: Defines traits for accessing directories like `done`, `failed_json_repairs`, `failed_items`, and `text_storage`.
- **Token Expansion and File Management**: Provides traits for handling token expansion paths and retrieving file paths related to failed repairs and items.

## Traits in the Crate

### BatchWorkspaceInterface

The `BatchWorkspaceInterface` trait bundles together all necessary operations for interacting with batch workspace files and directories. It includes the following individual traits:

- **GetInputFilenameAtIndex**: Provides a method for retrieving the input filename at a specific batch index.
- **GetOutputFilenameAtIndex**: Provides a method for retrieving the output filename at a specific batch index.
- **GetErrorFilenameAtIndex**: Provides a method for retrieving the error filename at a specific batch index.
- **GetMetadataFilenameAtIndex**: Provides a method for retrieving the metadata filename at a specific batch index.
- **GetDoneDirectory**: Returns the "done" directory in the workspace.
- **GetTokenExpansionPath**: Provides a method to get the token expansion path for a given token.
- **GetFailedJsonRepairsDir**: Returns the directory for failed JSON repairs.
- **GetFailedItemsDir**: Returns the directory for failed items.
- **GetTextStoragePath**: Returns the path for storing text associated with a given batch index.

## Usage

### Implementing the Interface

To implement these traits, create a structure (like `BatchWorkspace`) that provides the necessary methods for accessing the batch workspace files and directories. For example:

```rust
use batch_mode_batch_workspace_interface::{BatchWorkspaceInterface, GetInputFilenameAtIndex};

struct MyBatchWorkspace {
    workdir: PathBuf,
}

impl GetInputFilenameAtIndex for MyBatchWorkspace {
    fn input_filename(&self, batch_idx: &BatchIndex) -> PathBuf {
        self.workdir.join(format!("batch_input_{}.jsonl", batch_idx))
    }
}

impl BatchWorkspaceInterface for MyBatchWorkspace {}
```

### Accessing Batch Files

Once the interface is implemented, you can use it to access batch files and directories in the workspace:

```rust
let workspace = MyBatchWorkspace { workdir: "/path/to/workdir".into() };
let input_file = workspace.input_filename(&BatchIndex::Usize(1));
println!("Input file path: {:?}", input_file);
```

## License
This crate is licensed under the MIT License. See LICENSE for details.
