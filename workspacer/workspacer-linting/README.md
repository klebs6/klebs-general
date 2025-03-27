# workspacer-linting

`workspacer-linting` is a Rust crate designed to seamlessly execute linting tasks within a Rust workspace using `cargo clippy`. It leverages asynchronous operations to provide efficient and effective linting capabilities, ensuring that code adheres to predefined quality standards by treating compiler warnings as errors.

## Key Features

- **Asynchronous Linting**: Utilizes asynchronous Rust features to run linting operations without blocking the main thread.
- **Detailed Reporting**: Captures standard output and error streams into structured `LintReport` objects, enabling easy examination of linting results.
- **Error Handling**: Converts `cargo` process outputs into rich, domain-specific errors encapsulated in a `LintingError` type.

## Usage

This crate is intended for integration into existing Rust workspaces. Implement the `RunLinting` trait in your workspace type to use its functionalities. Ensure that `tokio` is correctly setup for asynchronous command execution.

Example:

```rust
#[async_trait]
impl RunLinting for YourWorkspaceType {
    type Report = LintReport;
    type Error = LintingError;
    async fn run_linting(&self) -> Result<Self::Report, Self::Error> {
        // Implementation goes here
    }
}
```

## Conceptual Overview

The crate defines a generalized `RunLinting` trait, providing flexibility in implementation. It's centered around executing `cargo clippy` commands in a non-blocking, resource-efficient manner, thereby facilitating the maintenance of high code quality.

## Requirements

- Rust 2024 Edition
- Tokio runtime for asynchronous execution

## License

This project is licensed under the MIT License. See the LICENSE file for details.