# workspacer-rebuild-or-test

**workspacer-rebuild-or-test** is a Rust crate designed to streamline the process of rebuilding and testing Rust projects located within crates or workspaces. This crate offers an asynchronous trait implementation, `RebuildOrTest`, to facilitate seamless integration with command runners, enabling automated build and test routines.

## Features

- **Asynchronous Execution:** Harnesses `async`/`await` to run `cargo build` and `cargo test` commands efficiently.
- **Error Handling:** Provides robust error handling through custom error types for builds and test failures.
- **Logging:** Emits log entries detailing build and test outcomes, aiding in diagnostics and workflow transparency.

## Interface
The primary interface is the asynchronous trait `RebuildOrTest`:

```rust
#[async_trait]
pub trait RebuildOrTest {
    type Error;
    async fn rebuild_or_test(&self, runner: &dyn CommandRunner) -> Result<(), Self::Error>;
}
```

This trait is implemented for both individual crates (`CrateHandle`) and entire workspaces (`Workspace`).

### Trait Implementations
- **CrateHandle:** Executes `cargo build` followed by `cargo test` within the crate directory.
- **Workspace:** Executes the same for all specified targets within a workspace.

## Usage
To employ the `workspacer-rebuild-or-test` crate, integrate it with your existing toolchain or CI/CD pipeline to ensure consistent build and test verification across your Rust projects.

## Prerequisites
- Rust 2024 Edition
- An appropriately configured `CommandRunner` interface to drive command execution.

## Installation
Add the crate to your `Cargo.toml`:

```toml
[dependencies]
workspacer-rebuild-or-test = "0.1.0"
```

## Contributing
Contributions towards enhancing the robustness and efficiency of this crate are encouraged. Please adhere to the standard Rust community guidelines for contributions.

## License
This project is licensed under the MIT License, permitting contributions provided with a clear notice in support of open source collaboration.
