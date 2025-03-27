# workspacer-check-crates-io

## Overview

`workspacer-check-crates-io` is a Rust crate that provides an asynchronous interface for verifying the publish status of a specific crate version on crates.io. This functionality is crucial for developers seeking to automate the release process while ensuring version uniqueness within the crates.io ecosystem.

## Features

- **Interface Trait**: Implements the `VerifyCrateVersionIsNotYetPublishedOnCratesIo` trait, offering an easy-to-use method to check if a crate version is already published.
- **Asynchronous Checking**: Employs asynchronous HTTP requests to crates.io for non-blocking verification.
- **Error Handling**: Robust error conversion from `WorkspaceError` to `CrateError`, ensuring clear diagnostics should issues arise.

## Usage

### Verify Trait

Implement the `VerifyCrateVersionIsNotYetPublishedOnCratesIo` for your crate handle to leverage its capabilities:

```rust
#[async_trait]
impl VerifyCrateVersionIsNotYetPublishedOnCratesIo for CrateHandle {
    type Error = CrateError;

    async fn verify_crate_version_is_not_yet_published_on_crates_io(&self) -> Result<(), Self::Error> {
        // Implementation...
    }
}
```

### Check Function

Extract the utility function to directly query if a certain version is published:

```rust
pub async fn is_crate_version_published_on_crates_io(crate_name: &str, crate_version: &semver::Version) -> Result<bool, WorkspaceError> {
    let url = format!("https://crates.io/api/v1/crates/{}/{}", crate_name, crate_version);
    // Make the HTTP request and handle the result...
}
```

## Technical Details

- **Crate Version Checking**: Connects to the crates.io API and verifies the presence of a specified crate version by analyzing HTTP status responses.
- **Dependencies**: Relies on external libraries like `reqwest` for HTTP requests and `async_trait` for trait convenience.

Ensure that your project is configured with the Rust 2024 edition for compatibility.

## Getting Started

To install this crate, add it to your `Cargo.toml` dependencies:
```toml
dependencies = [
    "workspacer-check-crates-io = \"0.1.0\""
]
```

To utilize the functionalities, implement the trait as described and invoke the check function where needed.

## License

This project is licensed under the MIT License. For more details, see the [LICENSE](LICENSE) file.
