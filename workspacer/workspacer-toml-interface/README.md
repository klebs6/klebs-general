# workspacer-toml-interface

`workspacer-toml-interface` is a Rust crate designed to facilitate seamless interactions with `Cargo.toml` files. This crate provides a suite of asynchronous and synchronous trait definitions, each encapsulating a distinct functionality concerning `Cargo.toml` management. Ideal for users looking to automate or streamline processes involving workspace configurations in Rust, this crate allows you to read, modify, clone, and verify repository configurations effectively.

## Features

- **Trait-based Architecture**: Implements a set of traits such as `CargoTomlInterface`, `WriteDocumentBack`, and more for clear abstractions.
- **Asynchronous Operations**: Employs Rust's `async_trait` for asynchronous functions, expanding possibilities for non-blocking tasks.
- **Dependency Management**: Easily update dependency versions using `UpdateDependencyVersionRaw`.
- **Integrity and Publishing Checks**: Comprehensive checks for `Cargo.toml` validity and integrity prior to publishing.
- **Edition and License Management**: Query and manage Rust edition and license type effectively.
- **Repository and Author Retrieval**: Retrieve and manage metadata such as authors and repository location.

## Usage

This crate is designed with extensibility and practical usage in mind, making it a fit for larger applications needing dynamic `Cargo.toml` management.

Incorporate the traits and functions in your project to enhance automated management of Rust project configurations. Use dependency management and metadata retrieval to create robust CI/CD pipelines, ensuring configuration correctness and ease of maintenance over complex Rust workspaces.

## Technical Details

This crate requires Rust edition 2024, taking advantage of modern language features.

Leverage `workspacer-toml-interface` for an authoritative mechanism to interact with the core metadata of Rust projects, enabling effective configuration manipulation and validation.