# Workspacer-Register

The `workspacer-register` crate is a comprehensive utility designed to ensure the registration of source files within a workspace, with specific focus on managing macro definitions and their seamless integration. Predominantly, it handles macros defined in the `x!` form, incorporating functionalities for deduplicating, assembling, and inserting macro blocks into source files efficiently.

## Overview

This crate offers several essential features:

- **Macro Detection and Parsing:** It identifies macros within `SourceFile` structures, specifically engineered to manage `x!` macros.
- **Macro Assembly:** Supports the creation of final top-block snippets, combining existing macros with new additions.
- **Duplicate Filtering:** Efficiently filters out duplicate macro stems from new insertions to prevent redundancies.
- **Async Source File Registration:** Utilizes asynchronous traits to ensure all source files within a workspace possess the requisite macro definitions.
- **Smart Insertion Point Logic:** Determines optimal insertion points for new macro blocks, taking into account existing structures and avoiding unnecessary whitespace disruptions.

## Technical Description

1. **Macro Processing and Management:**
   - `TopBlockMacro` and `ExistingXMacro`: Structs to encapsulate macro data with leading comments and range information.
   - Methods such as `build_top_block_for_no_imports_line`, handling complex assembly of macro snippets based on context and order.

2. **Async File Registration:**
   - The async trait `EnsureAllSourceFilesAreRegistered` enables concurrent operations over workspace crates, ensuring that source files are duly updated.

3. **Efficient Code Integration:**
   - Overhead minimization through deduplication using `filter_new_macros_for_duplicates`, preventing macro replication.
   - Utilizes `ra_ap_syntax` to parse and manipulate Rust source files with precise syntactic transformations.

## Getting Started

To utilize `workspacer-register`, include it in your `Cargo.toml` and implement the required traits in your workspace's custom logic. Leverage the provided utility functions to integrate and manage macros seamlessly across your project files.

## Use Case Example

```rust
use workspacer_register::{EnsureAllSourceFilesAreRegistered, Workspace};

#[tokio::main]
async fn main() {
    let workspace = Workspace::new();
    workspace.ensure_all_source_files_are_registered().await.unwrap();
}
```

This example demonstrates the initialization of the workspace and the invocation of the macro registration mechanism to enforce uniformity and completeness across source files. 

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.