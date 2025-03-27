# workspacer-check-private

`workspacer-check-private` is a Rust crate designed to ensure that your workspace crate is not flagged as private before proceeding with cargo publishing. Ideal for automated release workflows where ensuring the publishability of a crate is crucial.

## Overview

In various development scenarios, particularly in CI/CD pipelines, it is vital to programmatically ascertain that a crate is not private. By implementing the `VerifyCrateIsNotPrivate` trait, this crate provides asynchronous verification of crate visibility using the `async_trait` pattern.

## Getting Started

Integrate the crate in your existing project to utilize its verification capabilities. Ensure your environment is set up to handle Rust 2024 edition features if you aim for seamless integration.

### Example

```rust
#[async_trait]
impl VerifyCrateIsNotPrivate for CrateHandle {
    type Error = CrateError;

    async fn verify_crate_is_not_private(&self) -> Result<(), Self::Error> {
        // Illustration of how to handle verification
    }
}
```

## Technical Background

This crate leverages asynchronous programming patterns inherent in Rust, particularly suitable for high-performance and non-blocking concurrent applications.* It is crucial when handling task parallelism or I/O-bound operations often encountered in complex workspace environments.

## Contributing

We welcome contributions that improve the functionality and usability of this crate. Please adhere to Rust's coding conventions and submit your pull request.

## License

Distributed under the MIT License. See `LICENSE` for more information.
