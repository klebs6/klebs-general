# get-file-size

`get-file-size` is a Rust crate for efficiently determining the size of a file and counting the number of lines within a file asynchronously. This utility is particularly useful in systems where I/O operations need to be non-blocking, elevating performance in concurrent applications or environments where swift, streamlined file operations are essential.

## Features

- **Asynchronous File Size Retrieval**: Implement utilizing the `GetFileSize` trait, which employs asynchronous methods to fetch file metadata, ensuring minimal blocking during operations.

- **Line Counting**: An additional function, `count_lines_in_file`, provides the ability to count lines in a file asynchronously.

## Usage

Integrate `get-file-size` into your project by adding it as a dependency in your `Cargo.toml`:

```toml
[dependencies]
get-file-size = "0.1.0"
```

### Example

```rust
use get_file_size::GetFileSize;
use std::path::PathBuf;
use tokio;

#[tokio::main]
async fn main() -> Result<(), get_file_size::FileError> {
    let file_path = PathBuf::from("example.txt");

    // Obtain file size asynchronously
    let file_size = file_path.file_size().await?;
    println!("File size: {} bytes", file_size);

    // Count lines in file asynchronously
    let line_count = get_file_size::count_lines_in_file(&file_path).await?;
    println!("Line count: {}", line_count);

    Ok(())
}
```

## Contributing

Contributions, bug reports, and feature requests are welcome. Please check the [issues](https://github.com/klebs6/klebs-general/issues) page to get involved.

## License

This project is dual-licensed under the terms of the MIT license and the Apache License (Version 2.0).