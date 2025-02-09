# workspacer-interface

`workspacer-interface` is the core trait and error definitions crate for the Workspacer ecosystem. It defines the common interfaces that unify how workspaces and individual crates are managed. This crate provides a set of traits for:

- **Crate Handling:**  
  The `CrateHandleInterface` trait defines the operations required for working with a single crate (such as validating integrity, retrieving source files, and checking required files).

- **Cargo.toml Parsing & Validation:**  
  The `CargoTomlInterface` trait and its related helper traits standardize access to a crate’s manifest, ensuring required fields exist and are valid for both integrity and publishing.

- **Workspace Operations:**  
  The `WorkspaceInterface` trait bundles together common operations (cleanup, watching for changes, running tests with coverage, rebuilding, generating docs, linting, dependency analysis, etc.) required for managing multi-crate workspaces.

- **Unified Error Handling:**  
  A comprehensive set of error types is defined using `error_tree!`, allowing consistent error reporting across the Workspacer suite.

Built on top of Tokio and async_trait, all operations are designed to be asynchronous and nonblocking, making it well-suited for modern Rust development workflows.

## Features

- **Asynchronous Trait Methods:**  
  Uses `async_trait` to allow async methods in trait definitions for nonblocking operations.
  
- **Comprehensive Interface Definitions:**  
  Provides standardized interfaces for crate handling, Cargo.toml parsing, workspace management, and more.

- **Unified Error Management:**  
  Error types defined via `error_tree!` cover issues from I/O to validation and command execution, ensuring consistent error propagation across components.

- **Extensibility:**  
  Designed to be implemented by other components of the Workspacer ecosystem, enabling custom integrations and behavior extensions.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
workspacer-interface = "0.1.0"
```

## Usage

Implement the provided traits to integrate your workspace or crate handling with the Workspacer system. For example, you might implement `CrateHandleInterface` for your custom crate handler and then use `WorkspaceInterface` to orchestrate operations on an entire workspace.

```rust
use workspacer_interface::{CrateHandleInterface, WorkspaceInterface};

// Example: Implement your own CrateHandleInterface, or use an existing implementation.
```

For more details on available traits and error types, please consult the source or the documentation.

## Contributing

Contributions are welcome! Please see the [repository](https://github.com/klebs6/klebs-general) for guidelines on reporting issues and submitting pull requests.

## License

This project is dual-licensed under either the [MIT license](LICENSE-MIT) or the [Apache License, Version 2.0](LICENSE-APACHE), at your option.
