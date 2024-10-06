# workspacer

`workspacer` is a comprehensive Rust library for managing and validating workspaces and crates. It provides interfaces for ensuring that workspaces are clean, ready for publishing, and free from circular dependencies. The crate allows you to perform various actions such as running tests with coverage, generating documentation, and ensuring the integrity of crates.

## Features

- **Workspace Management**: Manage workspaces with multiple crates.
- **Crate Validation**: Ensure that each crate in the workspace has the necessary files and is ready for publishing.
- **Test Coverage**: Run tests with coverage and generate reports.
- **Circular Dependency Detection**: Automatically detect circular dependencies in workspaces.
- **Linting and Docs**: Run linting tools and generate documentation for the crates.

## Installation

To use `workspacer`, add it to your `Cargo.toml` dependencies:

```toml
[dependencies]
workspacer = "0.1"
```

## Quick Start

```rust
use workspacer::{Workspace, ValidateIntegrity, RunTestsWithCoverage, DetectCircularDependencies};
use tokio;

#[tokio::main]
async fn main() -> Result<(), workspacer::WorkspaceError> {
    // Initialize the workspace
    let workspace = Workspace::new("path_to_workspace").await?;

    // Validate workspace integrity
    workspace.validate_integrity()?;

    // Detect circular dependencies
    workspace.detect_circular_dependencies().await?;

    // Run tests with coverage
    let report = workspace.run_tests_with_coverage().await?;
    println!("Test coverage report: {:?}", report);

    Ok(())
}
```

## Workspace-Level Operations

### `WorkspaceInterface`

The `WorkspaceInterface` trait provides methods for managing and interacting with the entire workspace. Below are the key methods:

- **Get Crates**: Retrieve all the crates in the workspace.
  ```rust
  let crates = workspace.crates();
  ```

- **Number of Crates**: Get the number of crates in the workspace.
  ```rust
  let count = workspace.n_crates();
  ```

- **Cleanup Workspace**: Clean up any temporary files and artifacts.
  ```rust
  workspace.cleanup_workspace().await?;
  ```

- **Run Tests with Coverage**: Run tests across all crates in the workspace and generate coverage reports.
  ```rust
  let report = workspace.run_tests_with_coverage().await?;
  ```

- **Generate Dependency Tree**: Create a dependency tree for the workspace and output it in DOT format.
  ```rust
  let dot = workspace.generate_dependency_tree_dot().await?;
  println!("{}", dot);
  ```

- **Circular Dependency Detection**: Detect and report circular dependencies within the workspace.
  ```rust
  workspace.detect_circular_dependencies().await?;
  ```

- **Generate Documentation**: Automatically generate documentation for all crates.
  ```rust
  workspace.generate_docs().await?;
  ```

### Example: Running Tests with Coverage

```rust
use workspacer::{Workspace, RunTestsWithCoverage};

#[tokio::main]
async fn main() -> Result<(), workspacer::WorkspaceError> {
    let workspace = Workspace::new("path_to_workspace").await?;

    // Run tests with coverage
    let report = workspace.run_tests_with_coverage().await?;

    println!("Test coverage report: {:?}", report);

    Ok(())
}
```

## Crate-Level Operations

### `CrateHandleInterface`

The `CrateHandleInterface` provides methods for working with individual crates. It includes the following functionality:

- **Validate Crate Integrity**: Check that all required files (such as `Cargo.toml` and `README.md`) exist and are valid.
  ```rust
  crate_handle.validate_integrity()?;
  ```

- **Check for Source Files**: Ensure that the `src/` directory contains valid files such as `lib.rs` or `main.rs`.
  ```rust
  crate_handle.check_src_directory_contains_valid_files()?;
  ```

- **Check for README**: Ensure that the crate has a `README.md`.
  ```rust
  crate_handle.check_readme_exists()?;
  ```

- **Get Source Files with Exclusions**: Retrieve all source files in the `src/` directory, excluding certain files.
  ```rust
  let source_files = crate_handle.source_files_excluding(&["excluded_file.rs"]).await?;
  ```

- **Get Test Files**: Get all test files in the `tests/` directory.
  ```rust
  let test_files = crate_handle.test_files().await?;
  ```

### Example: Validating a Crate

```rust
use workspacer::{CrateHandle, ValidateIntegrity};

#[tokio::main]
async fn main() -> Result<(), workspacer::WorkspaceError> {
    let crate_handle = CrateHandle::new("path_to_crate").await?;

    // Validate the integrity of the crate
    crate_handle.validate_integrity()?;

    Ok(())
}
```

## Circular Dependency Detection

Use the `DetectCircularDependencies` trait to check for circular dependencies in your workspace.

### Example: Detecting Circular Dependencies

```rust
use workspacer::{Workspace, DetectCircularDependencies};

#[tokio::main]
async fn main() -> Result<(), workspacer::WorkspaceError> {
    let workspace = Workspace::new("path_to_workspace").await?;

    // Detect circular dependencies in the workspace
    workspace.detect_circular_dependencies().await?;

    Ok(())
}
```

## Test Coverage Reports

Generate test coverage reports using the `RunTestsWithCoverage` trait.

### Example: Running Tests with Coverage

```rust
use workspacer::{Workspace, RunTestsWithCoverage};

#[tokio::main]
async fn main() -> Result<(), workspacer::WorkspaceError> {
    let workspace = Workspace::new("path_to_workspace").await?;

    // Run tests with coverage
    let report = workspace.run_tests_with_coverage().await?;

    println!("Test coverage report: {:?}", report);

    Ok(())
}
```

## Linting and Documentation

Use the `RunLinting` and `GenerateDocs` traits to run linting tools and generate documentation for crates in the workspace.

### Example: Generating Documentation

```rust
use workspacer::{Workspace, GenerateDocs};

#[tokio::main]
async fn main() -> Result<(), workspacer::WorkspaceError> {
    let workspace = Workspace::new("path_to_workspace").await?;

    // Generate documentation for the workspace
    workspace.generate_docs().await?;

    Ok(())
}
```

## Error Handling

Errors are captured using the `WorkspaceError` and `CargoTomlError` types, which handle issues such as:
- Missing files (e.g., `Cargo.toml`, `README.md`)
- Invalid workspace configurations
- Circular dependencies
- Test coverage errors

## Contributing

Contributions are welcome! Feel free to open an issue or submit a pull request if you have suggestions or improvements.

## License

This project is licensed under the MIT License.
