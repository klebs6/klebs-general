# workspacer-mock

`workspacer-mock` is a utility crate in the Workspacer ecosystem designed for creating mock workspaces on the fly. This crate is especially useful for testing and development purposes—it allows you to specify crate configurations (via `CrateConfig`) and automatically generates a temporary workspace with a valid Cargo manifest, along with optional README, source, and test files for each member crate.

## Features

- **Asynchronous Workspace Creation:**  
  Leverages Tokio’s async file I/O to create directories, write files, and generate a workspace Cargo.toml without blocking your runtime.

- **Configurable Crate Setup:**  
  Use `CrateConfig` to determine if a mock crate should include a README, source files (in a `src/` directory), or test files (in a `tests/` directory).

- **Temporary Workspace Generation:**  
  Creates a unique temporary directory for the mock workspace (using a UUID), making it easy to run isolated tests or experiments.

## Installation

Add `workspacer-mock` to your `Cargo.toml` dependencies:

```toml
[dependencies]
workspacer-mock = "0.1.0"
```

## Usage

Below is an example of how you might create a mock workspace:

```rust
use workspacer_mock::create_mock_workspace;
use workspacer_crate::CrateConfig;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define configurations for your mock crates.
    let configs = vec![
        CrateConfig::new("crate_one").with_readme().with_src_files(),
        CrateConfig::new("crate_two").with_readme().with_src_files().with_test_files(),
    ];
    
    // Create the mock workspace.
    let workspace_path = create_mock_workspace(configs).await?;
    
    println!("Mock workspace created at: {}", workspace_path.display());
    
    Ok(())
}
```

This example creates a temporary workspace with two member crates—one with a README and source files, and the other with README, source, and test files. The generated workspace includes a valid `[workspace]` Cargo.toml, making it ready for further processing with the rest of the Workspacer tools.

## Contributing

Contributions are welcome! Please see the [repository](https://github.com/klebs6/klebs-general) for guidelines on reporting issues and submitting pull requests.

## License

This project is dual-licensed under either the [MIT license](LICENSE-MIT) or the [Apache License, Version 2.0](LICENSE-APACHE), at your option.
