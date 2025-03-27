# workspacer-errors

The `workspacer-errors` crate provides robust error handling for asynchronous operations within the `workspacer` workspace management environment. This crate facilitates effective error propagation and context management in concurrent ecosystems, ensuring nuanced error diagnostics and recovery strategies.

## Features

- **Tokio Error Handling**: Implements conversion from `tokio::task::JoinError` to custom error types `CrateError` and `WorkspaceError`, encapsulating asynchronous task execution failures.

- **IO Error Wrapping**: Converts `std::io::Error` into comprehensive `CrateError` and `WorkspaceError` variants, allowing for additional context to be attached to I/O operation failures, thus augmenting debugging and logging insights.

- **Metadata Error Inspection**: Provides a utility method to detect cyclic package dependencies in `CargoMetadataError`, enhancing the robustness of dependency management.

## Usage

```rust
use workspacer_errors::{CrateError, WorkspaceError};
use std::io;
use tokio::task::JoinError;

fn example_usage() -> Result<(), CrateError> {
    let join_error: JoinError = /* join operation */;
    let error: CrateError = join_error.into();
    // handle error
    Ok(())
}

fn io_error_handling() -> Result<(), CrateError> {
    let io_error: io::Error = /* I/O operation */;
    let error: CrateError = io_error.into();
    // manage the I/O error
    Ok(())
}
```

## Technical Details

The crate propagates errors across asynchronous contexts leveraging reference counting via `std::sync::Arc`, ensuring thread-safe handling of errors without incurring the cost of deep cloning.

Error conversions are strongly typed and contextual, allowing recursive introspection and meaningful error messages that are crucial in complex systems where debugging information fidelity is imperative.

## License

This crate is licensed under MIT License. See LICENSE file for details.