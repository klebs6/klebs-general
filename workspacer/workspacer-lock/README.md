# workspacer-lock

`workspacer-lock` is a Rust crate designed to facilitate the extraction and mapping of crate version data from a `Cargo.lock` file, a critical component within a Rust project for managing dependency versions. This crate provides an asynchronous function, `build_lock_versions`, that efficiently reads and interprets the `Cargo.lock` file from a designated root directory and constructs a comprehensive map of crate names to their respective version sets.

## Features

- **Asynchronous Processing**: Utilizes Rust's async capabilities to perform I/O operations, ensuring non-blocking reads of the `Cargo.lock` file.
- **Detailed Error Handling**: Implements robust error handling for missing file scenarios and lockfile parsing issues, allowing for precise debugging and fault isolation.
- **Data Structure Utilization**: Employs `BTreeMap` and `BTreeSet` for efficient storage and retrieval of crate-version mappings, preserving order and uniqueness.

## Usage

To use `workspacer-lock` in your project:

```rust
use workspacer_lock::build_lock_versions;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new("/path/to/project");
    let version_map = build_lock_versions(&root).await?;
    for (crate_name, versions) in version_map {
        println!("{}: {:?}", crate_name, versions);
    }
    Ok(())
}
```

## Crate Edition

This crate is compliant with the Rust 2024 edition, ensuring compatibility with the latest language features and improvements.

## Contributing

Contributions are welcome. Please adhere to the project's coding standards and ensure tests are included for new features or bug fixes.

## License

`workspacer-lock` is distributed under the MIT License. See [LICENSE](LICENSE) for more information.