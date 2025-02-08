# async-try-from

`async-try-from` provides a set of traits for defining asynchronous object creation and validation patterns in Rust. This crate builds on the [`async-trait`](https://crates.io/crates/async-trait) library to allow trait-based asynchronous construction, integrity checks, and more.

## Overview

1. **`AsyncTryFrom<X>`**: Asynchronously create a type `T` from some input `X`.
2. **`ValidateIntegrity`**: Perform synchronous integrity checks on an object.
3. **`AsyncCreateWithAndValidate<X>`**: Combines creation and validation into one asynchronous routine.
4. **`AsyncPathValidator`** and **`AsyncFindItems`**: Simplify common filesystem-related validations and item discovery.

## Usage

Below is a minimal working example demonstrating how to implement `AsyncTryFrom` and `ValidateIntegrity`, and then use the combined `AsyncCreateWithAndValidate` trait for easy creation-and-validation in a single call. 

Create a file, for example `examples/basic_usage.rs`, and paste in the following full Rust code:

```rust
mod basic_usage_example {
    use async_trait::async_trait;
    use async_try_from::{AsyncTryFrom, ValidateIntegrity, AsyncCreateWithAndValidate};
    use std::io;
    use tokio;

    // A simple struct that we will create asynchronously
    pub struct MyType;

    // Implement async creation from a String
    #[async_trait]
    impl AsyncTryFrom<String> for MyType {
        type Error = io::Error;

        async fn new(input: &String) -> Result<Self, Self::Error> {
            if input.is_empty() {
                Err(io::Error::new(io::ErrorKind::Other, "Input string is empty."))
            } else {
                Ok(MyType)
            }
        }
    }

    // Implement a basic validation check
    impl ValidateIntegrity for MyType {
        type Error = io::Error;

        fn validate_integrity(&self) -> Result<(), Self::Error> {
            // Add real integrity checks here if needed
            Ok(())
        }
    }

    // Demonstrate creating and validating our type in one step
    #[tokio::main]
    pub async fn run_example() -> Result<(), Box<dyn std::error::Error>> {
        let input = "Some input".to_string();
        let instance = MyType::new_and_validate(&input).await?;
        println!("Successfully created and validated MyType instance.");
        Ok(())
    }
}

fn main() {
    // In a real project, you could call:
    // basic_usage_example::run_example();
    // This example keeps main synchronous for demonstration.
    println!("Run 'cargo run --example basic_usage' to see the async creation and validation in action.");
}
```

Then run:

```bash
cargo run --example basic_usage
```

to see the async creation and validation in action.

## Features

- **Simple Trait Definitions**: Define async creation logic in a concise manner with `AsyncTryFrom`.
- **Optional Validation**: Use `ValidateIntegrity` to provide domain-specific checks on newly created objects.
- **Combined Flow**: `AsyncCreateWithAndValidate` merges creation and validation into a single method for convenience.
- **Filesystem Helpers**: `AsyncPathValidator` and `AsyncFindItems` can be used to handle path validation and item discovery.

## License

This project is licensed under the [MIT License](LICENSE).
