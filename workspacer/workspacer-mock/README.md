# Workspacer-Mock

`workspacer-mock` is a robust Rust crate designed to facilitate the creation and manipulation of mock Rust workspaces for development and testing purposes. Leveraging asynchronous programming principles, it allows users to programmatically generate workspaces with customizable crate configurations.

## Key Features
- **Asynchronous Handling**: Seamlessly create mock workspaces without blocking execution, leveraging Rust's async/await capabilities.
- **Configurable Crate Generation**: Specify options such as inclusion of README files, source directories, and test directories to suit development needs.
- **Error Simulation**: Simulate potential cargo publishing errors to ensure comprehensive testing.

## Technical Overview
Using `MockPath` and `MockCrateHandle`, developers can define the structure and readiness of their crates. The `create_mock_workspace` function generates a temporary workspace, executing tasks such as directory creation and Cargo.toml configuration, while allowing for the addition of basic files as needed. The crate supports the `ReadyForCargoPublish` trait to assess the readiness of the workspace for cargo operations.

## Getting Started
To utilize `workspacer-mock`, include it as a dependency in your project. Enjoy the convenience of automated workspace setup suited for both simple and complex testing scenarios.

```rust
// Example usage
use workspacer_mock::{create_mock_workspace, MockPath, MockCrateHandle};

// Define your crate configurations
let crate_configs = vec![]; // Populate with CrateConfig

// Create your mock workspace
let workspace_path = create_mock_workspace(crate_configs).await.expect("Failed to create workspace");
```

## Contribution and Licensing
Contributions are welcome. The project is dual-licensed under MIT and Apache-2.0. For more details, visit the [repository](https://github.com/klebs6/klebs-general).