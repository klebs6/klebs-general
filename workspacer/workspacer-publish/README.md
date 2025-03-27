# workspacer-publish

`workspacer-publish` is a Rust crate that provides an asynchronous interface for publishing Cargo packages. It is designed to handle the complexity of publishing crates from a Rust workspace, managing build order, dependency resolution, and registry configuration. This crate uses the `async_trait` pattern for asynchronous function execution, allowing operations to be non-blocking and efficient.

## Features

- **Asynchronous Publishing:** Utilizes async mechanisms to enable non-blocking I/O operations when publishing crates, enhancing performance and responsiveness.
- **Dependency Management:** Computes a topological order for publishing workspace crates, ensuring correct dependency resolution without cycles.
- **Dry-Run Capabilities:** Offers a dry-run mode to simulate a publish operation, which is essential for testing and CI/CD workflows without making actual changes.
- **Configurable Registries:** Supports publishing to different registries, with default support for both mock and official crates.io registries, contingent on environment configuration.

## Usage

To use this crate, implement the `TryPublish` trait for your workspace or crate handle, which requires an asynchronous implementation of the `try_publish` method. This method should handle logic for packaging, registration, and error management during the crate publication process.

Example:

```rust
#[async_trait]
impl TryPublish for MyCustomCrateHandler {
    type Error = MyCustomError;

    async fn try_publish(&self, dry_run: bool) -> Result<(), Self::Error> {
        // Implement your custom publishing logic here!
    }
}
```

## Installation

Add `workspacer-publish` to your `Cargo.toml` dependencies:

```toml
[dependencies]
workspacer-publish = "0.1.0"
```

## Environment Variables
- `USE_MOCK_REGISTRY`: Set to `1` to use a mock registry for publishing operations.

## License

`workspacer-publish` is MIT licensed.
