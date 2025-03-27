# workspacer-crate-mock

## Overview

`workspacer-crate-mock` is a Rust library providing a simulation framework for managing and interacting with a Rust crate's structure without engaging the actual filesystem. Designed for developers aiming to reliably test and simulate crate behaviors, this crate affords precision in simulating a crate's layout --- such as `src/main.rs`, `README.md`, and other pertinent files --- via a mock handle mechanism.

## Features

- Implements `CrateHandleInterface<P>` allowing for flexible mock behaviors.
- Simulates various scenarios, such as missing or invalid files, to enhance testing robustness.
- Capable of simulating a valid or invalid version perceived through the `MockCargoToml` behavior.
- Comprehensive state management using Rust's `Builder`, `MutGetters`, and `Getters` paradigms for efficient interaction and data handling.
- Backed by `AsyncMutex` for concurrent access and mutation of `MockCargoToml` instances.

## Use Cases

- Software tests requiring non-disruptive, replicable operations on mock crate structures.
- Experimentation with crate configurations in isolated environments.
- Dynamic responses to typical Rust project misconfigurations.

## Example Usage

Below is a basic example of how to instantiate a fully valid mock crate:

```rust
use workspacer_crate_mock::MockCrateHandle;

fn main() {
    let mock_crate = MockCrateHandle::fully_valid_config();
    // Engage your testing activities here
}
```

## Technical Requirements

- Rust 2024 edition is required for this crate.

## Contribution

We welcome contributions! Please ensure any pull requests adhere to the coding standards outlined in `CONTRIBUTING.md`. For issues or further discussions, refer to our GitHub repository.

---

Explore limitless possibilities of crate manipulation without the overhead of filesystem operations.