# get-file-size

`get-file-size` is an asynchronous utility crate that provides two key features for working with files:

- **Async File Size Retrieval:**  
  Retrieve the size of a file asynchronously by using the [`GetFileSize`] trait.

- **Asynchronous Line Counting:**  
  Count the number of lines in a file using the provided `count_lines_in_file` function.

It leverages [Tokio](https://tokio.rs) for async I/O, [async_trait](https://crates.io/crates/async_trait) to allow async trait methods, and [error_tree](https://crates.io/crates/error-tree) for ergonomic error handling.

## Features

- **Asynchronous File Metadata:**  
  Get the file size without blocking your async runtime.

- **Async Line Counting:**  
  Efficiently count the number of lines in a file, handling errors gracefully.

- **Ergonomic Error Handling:**  
  Custom error types make it easy to handle file I/O errors in your application.

## Installation

Add the following dependency to your `Cargo.toml`:

```toml
[dependencies]
get-file-size = "0.1.0"
```

## Usage

### Retrieve File Size

Implement the async file size retrieval via the [`GetFileSize`] trait:

```rust
use get_file_size::GetFileSize;
use tokio::fs;

#[tokio::main]
async fn main() {
    // Use any type that implements AsRef<Path>, for example a &str
    let file_path = "Cargo.toml";
    match file_path.file_size().await {
        Ok(size) => println!("File size: {} bytes", size),
        Err(e) => eprintln!("Error retrieving file size: {:?}", e),
    }
}
```

### Count Lines in a File

Use the `count_lines_in_file` function to count the number of lines in a file asynchronously:

```rust
use get_file_size::count_lines_in_file;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let file_path = PathBuf::from("Cargo.toml");
    match count_lines_in_file(&file_path).await {
        Ok(line_count) => println!("The file has {} lines", line_count),
        Err(e) => eprintln!("Error counting lines: {:?}", e),
    }
}
```

## License

This project is dual-licensed under either the [MIT license](LICENSE-MIT) or the [Apache License, Version 2.0](LICENSE-APACHE), at your option.

## Contributing

Contributions are welcome! Please check out the [repository](https://github.com/klebs6/klebs-general) for details.
