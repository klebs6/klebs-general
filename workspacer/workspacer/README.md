# Workspacer

## Overview
Workspacer provides a set of extended interfaces that facilitate operations on workspaces and crates within Rust projects. Implemented as traits `ExtendedWorkspaceInterface` and `ExtendedCrateInterface`, it simplifies and standardizes interactions, promoting code modularity and reusability.

## Features
- **Generic Interfaces**: Adapt these interfaces to different parameter types `P` and `T` for `ExtendedWorkspaceInterface`, maximizing the abstraction and flexibility of workspace operations.
- **Crate Focused**: With `ExtendedCrateInterface`, refine crate-level operations in a uniform and consistent manner.

## Usage
Include the crate in your Cargo.toml:
```toml
[dependencies]
workspacer = "0.6.0"
```

### Example
Implement these interfaces in your project to leverage their functionality:

```rust
use workspacer::{ExtendedCrateInterface, ExtendedWorkspaceInterface};

struct MyCrate;

impl<P> ExtendedCrateInterface<P> for MyCrate {
    // implement methods
}

struct MyWorkspace;

impl<P, T> ExtendedWorkspaceInterface<P, T> for MyWorkspace {
    // implement methods
}
```

## Licensing
Licensed under either of:
- MIT license
- Apache-2.0 license

## Contribution
Contributions are welcome. Please follow the guidelines outlined in the [GitHub repository](https://github.com/klebs6/klebs-general).

## Contact
Author: Klebs [tpk3.mx@gmail.com]
