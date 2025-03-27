# Workspacer-Bump

## Overview

`workspacer-bump` is a Rust crate designed for managing and incrementing semantic versions within a workspace. It provides automated version management by applying different release types such as Major, Minor, Patch, or Alpha with granularity on semantic versioning fields.

## Features

- **Semantic Versioning**: Supports major, minor, patch, and alpha version increments with pre-release and build metadata management.
- **Conforms with SemVer**: Aligned with Semantic Versioning 2.0.0 principles.
- **Async Traits**: Utilize asynchronous programming principles to handle version bumps in a non-blocking manner.
- **Workspace Downstream Management**: Automatically update dependent crates in a workspace recursively without manual intervention.
- **Robust Error Handling**: Error types describe potential failures, ensuring users can handle exceptions gracefully.

## Use Cases

`workspacer-bump` is suitable for projects that rely on a well-organized versioning mechanism, particularly those involving multiple interconnected crates that depend on each other, such as:

- Large Rust workspaces needing consistent version management.
- Automated pipelines requiring programmatic version bumping.
- Projects utilizing the Semantic Versioning method for version control.

## Basic Usage

```rust
use workspacer_3p::semver::Version;
use workspacer_bump::ReleaseType;

let mut v = Version::parse("1.2.3").unwrap();
ReleaseType::Major.apply_to(&mut v);
assert_eq!(v.to_string(), "2.0.0");
```

## Implementation
The crate defines various traits such as `Bump`, `BumpAll`, `BumpCrateAndDownstreams`, and `WorkspaceDownstreamExt` to facilitate version management across a workspace. Users can implement these traits in their crate structures to provide customized versioning logic.

### Traits
- **`BumpAll`**: Applies a version bump across all crates within a workspace.
- **`BumpCrateAndDownstreams`**: Targets a specific crate and its downstream dependencies for versioning adjustment.

### Traits Example
```rust
#[async_trait]
impl BumpAll for MyWorkspace {
    async fn bump_all(&mut self, release: ReleaseType) -> Result<(), Self::Error> {
        // Implementation
    }
}
```

## Macros
The crate includes a macro `gen_bump_all_for_workspace` to streamline the implementation of the `BumpAll` trait for both real and mock workspaces.

## Contributing
Bug reports and pull requests are welcome on GitHub. Each contribution is expected to adhere to the specified coding and contribution guidelines provided in our documentation.

## License
This project is licensed under the MIT License.