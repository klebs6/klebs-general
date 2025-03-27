# workspacer-register-internal-crate-in-prefix-group

A sophisticated Rust crate designed to manage and automate the registration of new internal crates into a pre-existing workspace-based prefix group. By manipulating the Cargo.toml and source files, it simplifies dependency management and modular organization within complex Rust projects.

## Overview

This crate provides a trait, `RegisterInPrefixGroup`, which allows seamless integration of new internal crates into a workspace, ensuring they are properly accounted for in the facade crate's architecture. This includes:

1. **Dependency Management**: Automatically adding the new crate as a path dependency in the facade crate's Cargo.toml under `[dependencies]`.
2. **Module Integration**: Ensuring that the new crate is publicly re-exported in the facade crate's `src/lib.rs`.

## Trait Details

### RegisterInPrefixGroup

- `async fn register_in_prefix_crate`
  - **Parameters**: `prefix_crate` and `new_crate` of type `H`, which must implement the `CrateHandleInterface`.
  - **Error Handling**: Returns a `Result` with potential errors encapsulated in `WorkspaceError`.

### Implementation Highlights

- **Path Calculation**: Dynamically computes the relative path between the prefix and new crate directories.
- **Cargo.toml Editing**: Utilizes `toml_edit` to read, modify, and write dependency configurations.
- **Source Code Integration**: Appends and manages appropriate `pub use` statements in the source code.

This crate is specifically tailored for projects that utilize asynchronous programming paradigms, enabled by the `async_trait` dependency, and requires a foundation in robust error handling and concurrent computing practices.

## Application

Ideal for building scalable and modular Rust applications that require organized and manageable project structuring, especially those that follow the prefix group pattern.

## Installation

To use this crate, add it as a dependency in your project's `Cargo.toml`, and implement the `RegisterInPrefixGroup` trait for your project-specific types.