# workspacer-format-imports

`workspacer-format-imports` is a Rust crate that provides a comprehensive framework for syntactic refinement of Rust code, specifically focusing on the organization of import statements. It aims to streamline and organize import declarations across a Rust crate or workspace for enhanced readability and maintainability.

## Features

- **Syntactic Analysis**: Parses the context of `src/imports.rs` using Rust Analyzer's abstract syntax tree (RA-AP-syntax).
- **Use Statement Aggregation**: Collects all `use` statements, allowing the detection and extraction of syntax elements.
- **Prefix Grouping**: Groups `use` statements by their prefixes, converging identical paths within braces for succinctness:
  ```rust
  pub(crate) use std::collections::HashMap;
  pub(crate) use std::collections::HashSet;
  // becomes
  pub(crate) use std::collections::{HashMap, HashSet};
  ```
- **Alphabetical Sorting**: Ensures all grouped `use` statements are organized in alphabetical order.
- **Comment Preservation**: Retains inline and block comments adjacent to `use` statements during reorganization.
- **Async Operations**: Integrates with asynchronous tasks to facilitate non-blocking operations.
- **Robust Structuring**: Provides detailed constructs for managing syntax nodes and fine-tuning comment handling.

## Usage

Implement the `SortAndFormatImports` trait to utilize the organization and formatting capabilities:
```rust
#[async_trait]
impl SortAndFormatImports for CrateHandle {
    type Error = CrateError;
    async fn sort_and_format_imports(&self) -> Result<(), Self::Error> {
        // implementation details
    }
}
```

Integrate the crate into a Rust workspace to standardize import formatting and bolster code clarity.

## Advantages

- **Improves Code Clarity**: Minimizes redundant code through logical grouping and sorting.
- **Automates Refactorings**: Reduces human error in manual organizing of `use` statements.
- **Supports Large Workspaces**: Handles compliances and complexities of a full crate ecosystem.

Ensure you have the Rust 2024 edition enabled to exercise all features of the crate effectively.

## License

`workspacer-format-imports` is available under the MIT License.