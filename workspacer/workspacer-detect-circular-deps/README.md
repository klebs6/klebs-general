# Workspacer Detect Circular Deps

workspacer-detect-circular-deps is a Rust crate designed to generate and analyze dependency trees within a Rust workspace, identifying any circular dependencies. It leverages the `cargo_metadata` library to obtain detailed project metadata and constructs a directed graph representing each package and its dependencies.

## Interface Overview

The crate is structured around two primary asynchronous traits:

### GenerateDependencyTree
- **generate_dependency_tree**: Constructs a directed graph of the dependency tree from workspace metadata.
- **generate_dependency_tree_dot**: Provides a representation of the dependency tree in DOT format, useful for visualization purposes.

### DetectCircularDependencies
- **detect_circular_dependencies**: Utilizes workspace metadata to identify circular dependencies and handles specific cyclic dependency errors accordingly.

## Directed Graph Representation

The dependency structure is modeled using `WorkspaceDependencyGraph`, a directed graph (`DiGraph<String, ()>`) where nodes represent packages and directed edges represent dependencies.

### Application and Use

Users can implement these traits to automate the detection of circular dependencies, facilitating more robust workspace configuration and improved dependency management in Rust projects. By visualizing dependencies, developers can better understand and optimize their project's architecture.

This crate is especially useful in scenarios involving complex dependency networks within Rust workspaces, aiding in the prevention of potential build and runtime errors due to cyclic dependencies.