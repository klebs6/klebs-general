# workspacer-name-all-files

`workspacer-name-all-files` is a Rust crate that facilitates the management of Rust source files (.rs) within complex workspace directories. This crate provides utilities to recursively scan directories, identify .rs files, and ensure consistent file markers across the codebase.

## Features

- **Asynchronous Path Traversal**: Leveraging Rust's async capabilities, the crate allows for efficient directory scanning and file manipulation.
- **File Marker Management**: Automatically removes legacy markers and inserts new standardized markers into every .rs file.
- **Workspace Compatibility**: Designed to integrate seamlessly with Rust workspaces, supporting custom crate handles and potential directory accessibility issues.
- **Error Handling**: Comprehensive error reporting tailored for both I/O operations and marker management.

## Usage

To utilize `workspacer-name-all-files`, your workspace must implement the `NameAllFiles` trait, enabling each crate within your workspace to participate in the file naming process.

```rust
use workspacer_name_all_files::{NameAllFiles, gather_rs_files_recursively};

#[async_trait]
impl NameAllFiles for MyWorkspace {
    type Error = MyWorkspaceError;
    async fn name_all_files(&self) -> Result<(), Self::Error> {
        // Custom implementation here
    }
}
```

### Example

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workspace = MyWorkspace::new();
    workspace.name_all_files().await?;
    Ok(())
}
```

## Contributions

Contributions are welcome! Please adhere to the code of conduct outlined in the repository documentation.

## License

Distributed under the MIT License.