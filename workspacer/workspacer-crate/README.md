# workspacer-crate

**workspacer-crate** is a comprehensive utility for managing Rust workspaces with refined asynchronous handling and intricate management of workspace constructs. This crate is tailored for developers who require detailed control over crate initialization, integrity validation, version management, and more.

## Features

- **Crate Initialization**: Efficiently initialize crates within a workspace, complete with optional README, source files, and test files.
  
- **Asynchronous Operations**: Utilize asynchronous mechanisms to manage concurrent operations, ensuring thread-safe access and modification of the `Cargo.toml` and associated paths.

- **Version and Integrity Management**: Safeguard version consistency and workspace integrity with automated checks for required files such as `Cargo.toml` and source files.

- **Serialize and Deserialize Support**: Offers serialization for crate handles, facilitating network transmission or persistent storage.

## Usage

Incorporate `workspacer-crate` into your Rust project to streamline workspace management:

```rust
use workspacer_crate::CrateHandle;

async fn main() {
    // Example usage of CrateHandle
    let crate_handle = CrateHandle::new("/path/to/crate").await.unwrap();
    let version = crate_handle.version().unwrap();
    println!("Crate version: {}", version);
}
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
workspacer-crate = "0.5.0"
```

## Contributions

Contributions are welcome! Please look at the [issues](https://github.com/klebs6/klebs-general/issues) on GitHub.

## License

This project is licensed under the MIT OR Apache-2.0 licenses.