# workspacer-config

`workspacer-config` is a Rust crate designed to handle configuration data stored in `.ws` directories. Utilizing this crate facilitates file management and configuration settings for local or global workspaces, intended for various purposes, such as readme writing or test upgrading.

## Usage

To integrate `workspacer-config` into your project, leverage its two primary structures: `WorkspacerConfig` and `WorkspacerDir`.

### WorkspacerConfig

This struct represents the configuration data that resides within `workspacer-config` files, encoded in TOML format. It is solely responsible for dealing with the data itself and does not involve directory creation logic.

- **authors**: List of authors.
- **rust_edition**: The Rust edition being used.
- **license**: License information.
- **repository**: Associated repository link.

### WorkspacerDir

`WorkspacerDir` abstracts the operation of `.ws` directories. This struct differentiates between the directory itself and the configuration file within, and offers various utility methods:

- **Create/Ensure Subdirectories**: Implement subdirectories like `readme-writer-workspace`.
- **Remove Subdirectories**: For cleanup processes.
- **Load Configurations**: Asynchronously load configuration data, with fallbacks if needed.

### Example

To manage a `.ws` directory:

```rust
let local_dir = WorkspacerDir::local();
local_dir.ensure_dir_exists()?;
let config = local_dir.load_or_create_config_async().await?;
```

## Async Operations

Most directory and file operations in `workspacer-config` are asynchronous, allowing for efficient non-blocking I/O. This design aligns with modern Rust practices, providing robust functionality for extensive workspace infrastructure.

## Error Handling

The crate utilizes `WorkspacerFallbackError` to encapsulate potential error cases throughout the workspace directory and configuration operations.

## Integration

Integrate easily into existing Rust projects by importing this crate and using its considerate APIs for workspace directory management.