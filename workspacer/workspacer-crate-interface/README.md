# workspacer-crate-interface

`workspacer-crate-interface` is a Rust library designed to interface with Rust crates, providing comprehensive traits and implementations for querying and manipulating a crate's file system context.

## Overview
This crate offers a suite of traits that streamline interaction with various directories and files within a Rust project, such as `Cargo.toml`, source files, README, and test directories. The provided interfaces are especially useful for developers aiming to programmatically interact with Rust projects, whether for analysis, validation, or tooling purposes.

## Key Features
- **Asynchronous and Synchronous Operations:** Adaptable to both synchronous and asynchronous contexts, allowing you to implement file operations efficiently.
- **Comprehensive Trait Coverage:** Traits cover essential operations, including reading files, accessing `Cargo.toml`, identifying test directories, and excluding files.
- **Error Handling:** Consistent use of `Result` types to handle file-related errors robustly.

## Usage
Integrate it into your project by implementing the desired traits for your specific use cases. It is especially suited for creating tooling around Rust projects where accessing and manipulating project metadata and directory structure is necessary.

### Sample Implementation Structure
Implement these traits and use provided implementations to lock, read, and write essential files and directories:

- **CrateHandleInterface**: Base trait for crate handling.
- **HasCargoToml**: Access `Cargo.toml` using async locks.
- **ReadFileString**: Fetch file contents.
- **GetTestFiles**: Retrieve test files using locks.
- **CheckIfReadmeExists**: Verify presence of README.
- **GetSourceFilesWithExclusions**: Manage source files with exclusion logic.

## Edition
This crate uses Rust edition 2024.

## License
This project is licensed under the MIT License. See the LICENSE file for more details.
