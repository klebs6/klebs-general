# workspacer-scan-for-prefix-groups

A robust Rust crate providing asynchronous utilities for identifying and validating prefix-based crate groups within a workspace. Built on asynchronous traits and leveraging advanced concurrency paradigms, this crate is suitable for sophisticated crates management that requires the articulation of cohesive structures based on crate prefixes. 

## Features

- **Asynchronous Cohesion Validation**: Ensure your prefix groups maintain integrity using our `ValidatePrefixGroupCohesion` trait.
  - Checks for missing facade or 3p crates.
  - Ensures member crates depend on 3p crates where applicable.
  - Validates facade crates' re-export of group members.

- **Efficient Prefix Group Scanning**: Use the `ScanPrefixGroups` trait to efficiently scan your workspace and delineate coherent crate structures by leveraging prefix logic.
  - Implements a 'longest facade' logic ensuring comprehensive group structuring.
  - Constructs `PrefixGroup` snapshots ensuring immutable real-time data consistency across concurrent operations.

## Usage

Embed this crate into your workspace management workflow. Implement the required traits on your workspace entity to allow for seamless prefix group scanning and validation, therefore augmenting the integrity and cohesion of grouped crates within large scale Rust projects.

### Example

```rust
use workspacer_scan_for_prefix_groups::{ValidatePrefixGroupCohesion, ScanPrefixGroups};

#[derive(Debug, Clone)]
struct MyCrateHandle;
impl CrateHandleInterface<PathBuf> for MyCrateHandle {}

struct MyWorkspace;
impl ScanPrefixGroups<PathBuf, MyCrateHandle> for MyWorkspace {
    type Error = WorkspaceError; 
    // Example implementation
}

#[tokio::main]
async fn main() {
    let workspace = MyWorkspace;
    // Conduct scanning
    let groups = workspace.scan().await.unwrap();
    // Validate group cohesion
    workspace.validate_prefix_group_cohesion().await.unwrap();
}
```

## Technical Requirements

This crate requires the 2024 Rust edition for enhanced async concurrency and trait implementations. 

## Contributing

Contributions to enhance functionality or increase compatibility are welcome. Please submit PRs or issues to the repository for review.
