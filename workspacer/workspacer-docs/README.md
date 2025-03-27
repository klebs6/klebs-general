# workspacer-docs

`workspacer-docs` is a Rust crate providing an asynchronous interface to generate documentation for Rust workspaces using `cargo doc`. It is designed for seamless integration into workflows requiring programmatic documentation generation and error handling. This crate leverages the `async_trait` library to support asynchronous operations, ideal for modern, non-blocking Rust applications.

## Features

- **Asynchronous Execution**: Utilize Rust's async capability to generate documentation without blocking the current thread.
- **Error Handling**: Comprehensive handling of I/O and command execution errors, providing feedback from the `cargo doc` process.
- **Integration with Workspaces**: Easily integrates into projects structured as Cargo workspaces.

## Usage

To use `workspacer-docs`, implement the `GenerateDocs` trait on your workspace structure. The trait requires defining one method, `generate_docs`, which will invoke `cargo doc` in the context of your workspace directory.

### Example

```rust
use async_trait::async_trait;

#[async_trait]
impl GenerateDocs for MyWorkspaceType {
    type Error = MyErrorType;

    async fn generate_docs(&self) -> Result<(), Self::Error> {
        // Your logic to generate docs
    }
}
```

## Requirements

- Rust 2024 edition
- Cargo for building documentation

## License

This project is licensed under MIT License.