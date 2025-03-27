# workspacer-show-dependency-tree

`workspacer-show-dependency-tree` is a Rust crate designed for software developers and engineers involved in managing complex projects. This crate efficiently generates and displays a comprehensive dependency tree for Rust workspaces, facilitating seamless dependency management and insight into project structure.

## Features

- **Visualize Dependencies**: Generate clear and understandable tree structures of your workspace dependencies.
- **Support for Workspaces**: Explicitly designed to handle Rust's workspace feature, ideal for multi-crate projects.
- **Customizable Output**: Offers options to display dependency trees in various formats aiding different project requirements.
- **Performance Oriented**: Leverages Rust's efficient concurrency features to deliver fast tree generations even for large projects.

## Installation

To include this crate in your project, add the following to your `Cargo.toml`:

```toml
[dependencies]
workspacer-show-dependency-tree = "0.1.0"
```

## Usage

```rust
use workspacer_show_dependency_tree::generate_tree;

fn main() {
    let tree = generate_tree("path/to/workspace");
    println!("{}", tree);
}
```

## Background

Understanding and managing dependencies is crucial for maintaining robust and scalable software applications. Dependency trees offer insight into potential bottlenecks and highlight deeply nested dependencies which may introduce complexity or technical debt.

## Contribution

Contributions to improve this crate are welcome. Please see the repository for guidance on how to contribute.

## License

This project is licensed under the MIT License.