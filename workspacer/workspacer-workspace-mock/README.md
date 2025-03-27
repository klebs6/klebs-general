# workspacer-workspace-mock

## Overview

`workspacer-workspace-mock` provides a fully configurable mock framework tailored for simulating and testing Rust workspace environments. This crate is ideal for developers needing to replicate various workspace scenarios — from valid setups to specific error states — without the need for actual file system deployment.

## Features

- **Mock Workspace Simulation**: Easily mimic different workspace configurations using the `MockWorkspace` structure, like missing `Cargo.toml` files or simulating a non-workspace scenario.
- **Global Registration**: Register mock workspaces in a global registry, allowing quick retrieval and testing consistency across asynchronous tasks.
- **Crate Management**: Utilize the ability to dynamically add, find, and validate mocked crates within a workspace.
- **Integrity Checks**: Leverage built-in functionality for simulating integrity validation scenarios.
- **Concurrency Support**: Integration with asynchronous programming and data protection using `AsyncMutex`.

## Usage

Begin by constructing a `MockWorkspace` using the `fully_valid_config` method for a comprehensive setup that simulates a realistic workspace under controlled parameters.

```rust
use workspacer_workspace_mock::MockWorkspace;

let mock_workspace = MockWorkspace::fully_valid_config();
mock_workspace.register_in_global().await;
```

Harness additional methods like `find_crate_by_name` and `validate_integrity` to further manipulate and interrogate your mock environment.

## Advanced Configuration

The structure supports numerous combinations and offers flexible controls for edge-case testing.

- **Simulation Controls**: Tweak various booleans such as `simulate_missing_cargo_toml` to guide the simulated environment's behavior.
- **Custom Handlers**: Opt for different type constraints that comply with `CrateHandleInterface` to customize gone crate interactions.

## Supported Rust Version

This crate is designed for the Rust 2024 edition, ensuring compatibility with the latest language advancements.

## Contribution

Contributions are welcome. For major changes, please open an issue first to discuss what you would like to change.

## License

This project is licensed under the MIT License.

