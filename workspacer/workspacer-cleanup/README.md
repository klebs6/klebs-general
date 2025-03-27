# Workspacer-Cleanup

`workspacer-cleanup` is a Rust crate designed to streamline development workflows by efficiently purging unnecessary files and directories within a workspace. Employing asynchronous operations, this crate helps maintain a tidy project environment, crucial for optimal performance during compilation cycles and versioning tasks.

## Features

- **Asynchronous Execution**: Implements Rust's async paradigms to ensure non-blocking file and directory operations.
- **Targeted Cleanup**: Specifically focuses on auto-generated files and directories such as `target/` and `Cargo.lock` to prevent clutter.
- **Error Handling**: Robust error handling with custom `WorkspaceError` enumerations for both file and directory removal operations.
- **Composable Interface**: Easy to integrate with existing Rust workspaces through the implementation of the `CleanupWorkspace` trait.

## Usage

 Incorporate `workspacer-cleanup` into your project by implementing the `CleanupWorkspace` trait within your workspace structure. Utilize the `cleanup_workspace` method to perform the cleanup operation asynchronously.

```rust
#[async_trait]
pub trait CleanupWorkspace {
    async fn cleanup_workspace(&self) -> Result<(), WorkspaceError>;
}

// Example implementation
impl<P, H: CrateHandleInterface<P>> CleanupWorkspace for Workspace<P, H> where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait {
    async fn cleanup_workspace(&self) -> Result<(), WorkspaceError> {
        // Cleanup logic here
    }
}
```

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
workspacer-cleanup = "0.1.0"
```

## Compatibility

This crate requires Rust edition 2024 and is intended for systems where asynchronous file operations are optimal.

## Contributing

Contributions are welcome! Please adhere to the standard Rust coding practices, and ensure all changes are covered with appropriate tests.