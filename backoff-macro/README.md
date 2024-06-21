# backoff_macro

`backoff_macro` is a Rust procedural macro that adds retry logic with exponential backoff to asynchronous functions. This crate leverages the `backoff` crate to handle retries and allows users to simply annotate their functions with `#[backoff]` to enable this functionality.

## Features

- Automatically retries asynchronous functions on failure.
- Uses exponential backoff to space out retries.
- Simple and easy-to-use attribute macro.

## Installation

Add `backoff_macro` to your `Cargo.toml`:

```toml
[dependencies]
backoff_macro = "0.1.0"
backoff = "0.3.0"
tokio = { version = "1", features = ["full"] }
```
## Usage

To use the `#[backoff]` macro, simply annotate your asynchronous function with `#[backoff]`. The function should return a `Result<T, E>` type, where `E` implements the `From<E>` trait for `backoff::Error<E>`.

### Example

Here's a basic example demonstrating how to use the `#[backoff]` macro:

```rust
use backoff_macro::backoff;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::error::Error;
use tokio::main;

#[backoff]
async fn dummy_function(attempts: Arc<AtomicUsize>) -> Result<(), &'static str> {
    let current_attempts = attempts.fetch_add(1, Ordering::SeqCst);
    if current_attempts < 2 {
        Err("error")
    } else {
        Ok(())
    }
}

#[main]
async fn main() -> Result<(), Box<dyn Error>> {
    let attempts = Arc::new(AtomicUsize::new(0));

    let result = dummy_function(attempts.clone()).await;

    match result {
        Ok(_) => println!("Function succeeded after retries"),
        Err(e) => println!("Function failed after retries: {}", e),
    }

    Ok(())
}
```

### Explanation

1. **Annotation**: Use `#[backoff]` to annotate the asynchronous function you want to apply retry logic to.
2. **Retry Logic**: The macro will wrap the function body with retry logic using the `backoff::future::retry` method.
3. **Return Type**: Ensure the function returns a `Result<T, E>` type.

## Contributing

Contributions are welcome! If you find a bug or have a feature request, please open an issue on the GitHub repository.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

