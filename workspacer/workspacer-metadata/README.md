# workspacer-metadata

`workspacer-metadata` is a Rust crate designed to facilitate seamless retrieval of Cargo package metadata in an asynchronous context. Leveraging Rust's `async_trait` and Tokio's asynchronous task-parallelism framework, this crate is ideal for systems requiring non-blocking operations in Rust workspaces.

## Features

- **Asynchronous Metadata Retrieval**: Extract detailed Cargo package metadata without blocking the execution of other processes.
- **Interoperability**: Integrates smoothly with existing Cargo system paths through the `Workspace` abstraction.
- **Error Handling**: Offers robust error management via custom error types for precise error classification and handling.

## Getting Started

```rust
use workspacer_metadata::GetCargoMetadata;

// Define your workspace with proper path and handle implementations.
let my_workspace = Workspace::new().expect("Workspace creation failed");

// Fetch metadata asynchronously
let metadata = my_workspace.get_cargo_metadata().await.expect("Failed to get metadata");
```

## Technical Details

This crate is built on the `async` and `await` constructs of modern Rust, relying on Tokio's task management system (`spawn_blocking`) to execute potentially blocking operations in a non-blocking way. It employs a trait-based API allowing any compatible workspace type to retrieve metadata seamlessly.

To utilize this crate, ensure your environment is compliant with Rust's 2024 edition standards.