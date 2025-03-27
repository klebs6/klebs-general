# workspacer-pin

`workspacer-pin` is a Rust crate designed for resolving wildcard dependencies within a workspace, focusing on enhancing package management via precise dependency pinning. It is crafted to pinpoint all dependencies denoted with wildcard characters (`"*"`) and replace them with specific versions derived from local or lockfile sources.

## Features

- **Automated Dependency Resolution**: Utilizes provided traits to systematically replace wildcard dependencies with actual versions.
- **Local Path Version Sourcing**: For dependencies specified by local paths, actual version numbers are pulled directly from corresponding `Cargo.toml` files.
- **Lockfile Reference**: When dealing with workspace dependencies without explicit local paths, it falls back on versions listed in the lockfile.
- **Nested Dependency Handling**: Traverses complex nested tables in `Cargo.toml` to ensure all wildcard instances are addressed.

## Usage

Integrate `workspacer-pin` into your workspace as follows:

- Implement `PinAllWildcardDependencies` and `PinWildcardDependencies` for your structures, ensuring error handling congruence.
- Leverage asynchronous functions such as `pin_wildcard_dependencies_in_table` and `fix_nested_tables` to walk through dependency structures and pin versions appropriately.

### Example

```rust
use workspacer_pin::{PinAllWildcardDependencies, PinWildcardDependencies};

async fn manage_dependencies(workspace: &mut Workspace) -> Result<(), WorkspaceError> {
    workspace.pin_all_wildcard_dependencies().await
}
```

## Technical Specifics

The crate utilizes advanced Rust features including async/await patterns and trait implementations. The `LockVersionMap` is a primary data structure, offering efficient lookup of available versions with its combination of `BTreeMap` and `BTreeSet`.

## Audience

This crate is aimed at developers managing Rust workspaces who require robust methods to reconcile dependencies, especially for large or complex projects featuring a myriad of localized and external dependencies.