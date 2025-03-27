# workspacer-add-internal-dep

A Rust crate that provides functionality to facilitate the addition of internal dependencies between crates within a Cargo workspace. This crate is especially tailored for developers who need to efficiently manage and automate dependency inclusion, ensuring that complex project structures with nested dependencies are correctly handled.

## Overview

The primary composition of the crate revolves around the `AddInternalDependency` trait, designed to be implemented for a `Workspace` type. This allows users to manage dependencies dynamically in an asynchronous context. It utilizes `toml_edit` to modify the project's Cargo.toml file and handles potential I/O errors with precise error reporting, ensuring robust operational behavior.

The crate also updates the target crate's `imports.rs` file to propagate new dependencies, maintaining coherence across the workspace.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
workspacer-add-internal-dep = "0.1.0"
```

## Usage

```rust
use workspacer-add-internal-dep::AddInternalDependency;
// Assume Workspace and other structs are already defined
// let workspace: Workspace<_, _> = ...;
// workspace.add_internal_dependency(&target_crate, &dependency_crate).await.unwrap();
```

### Features

- Asynchronous execution model ensures non-blocking dependency management.
- Uses `toml_edit` for safe and concurrent manipulation of TOML files.
- Comprehensive error handling with descriptive context.

Please consult the documentation for a complete guide on function signatures and error types.

## License

Available under the MIT License. See the LICENSE file for more information.