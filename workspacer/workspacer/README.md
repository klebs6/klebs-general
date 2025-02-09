# Workspacer

Workspacer is an asynchronous workspace management library for Rust projects. It bundles a wide range of utilities for automating common tasks in a multi-crate workspace, such as:

- **Watching and Reloading:**  
  Monitor file changes (e.g. in `src/` or `Cargo.toml`) and automatically trigger rebuilds or tests.

- **Cleanup:**  
  Remove build artifacts and lock files (like `target/` and `Cargo.lock`) to keep your workspace tidy.

- **Test Coverage:**  
  Run tests with code coverage using `cargo tarpaulin` and generate detailed coverage reports.

- **Metadata Retrieval:**  
  Fetch Cargo metadata asynchronously to drive further analysis.

- **Workspace Analysis:**  
  Compute metrics (file sizes, lines of code, etc.) across all crates in the workspace.

- **Documentation Generation:**  
  Automatically generate documentation for the entire workspace by running `cargo doc`.

- **Rebuild or Test:**  
  Wrap common Cargo commands (build/test) into a unified interface to streamline your workflow.

- **Dependency Tree Generation:**  
  Create a dependency graph (and export it in DOT format) for your workspace.

- **Circular Dependency Detection:**  
  Identify cyclic dependencies among workspace crates.

- **Linting:**  
  Run `cargo clippy` to enforce code quality and catch potential issues.

Built on top of Tokio and other async utilities, Workspacer lets you integrate all these features into your development workflow without blocking your async runtime.

## Features

- **Asynchronous Operations:** All major operations (watching, metadata retrieval, rebuilding, etc.) are implemented asynchronously.
- **Extensible Traits:** Customize behavior by implementing or extending the provided traits.
- **Integrated Tooling:** Wraps and orchestrates common Cargo commands for a unified developer experience.
- **Comprehensive Analysis:** Provides both file-level and workspace-level insights to help you maintain a healthy codebase.

## Installation

Add Workspacer to your `Cargo.toml`:

```toml
[dependencies]
workspacer = "0.1.0"
```

## Usage

Below is a brief example that shows how you might initialize a workspace and perform some common operations:

```rust
use workspacer::{Workspace, WorkspaceInterface};
use tokio::runtime::Runtime;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize a workspace from a given path
    let workspace_path = PathBuf::from("/path/to/your/workspace");
    let workspace = Workspace::new(&workspace_path).await?;
    
    // Validate workspace integrity
    workspace.validate_integrity()?;
    
    // Run tests with coverage and print the result
    let coverage_report = workspace.run_tests_with_coverage().await?;
    println!("Total coverage: {}%", coverage_report.total_coverage());
    
    // Generate documentation for the workspace
    workspace.generate_docs().await?;
    
    // Clean up temporary or build files in the workspace
    workspace.cleanup_workspace().await?;
    
    // Optionally, trigger rebuild or testing on file changes
    // workspace.watch_and_reload(...).await?;
    
    Ok(())
}
```

## Contributing

Contributions are welcome! Please see the [repository](https://github.com/klebs6/klebs-general) for details on how to contribute.

## License

This project is dual-licensed under either the [MIT license](LICENSE-MIT) or the [Apache License, Version 2.0](LICENSE-APACHE), at your option.
