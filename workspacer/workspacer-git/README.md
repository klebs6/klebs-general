# workspacer-git

`workspacer-git` is a Rust library designed to ensure that your Git workspaces are in a clean state before proceeding with operations that require a pristine environment. By leveraging async traits, it facilitates efficient integration into modern asynchronous workflows.

## Overview

The core functionality is encapsulated in the `EnsureGitClean` trait, which enforces validations on Git working directories across workspaces or single crates. This trait assumes that `git` is available in the runtime environment and that directories should either have no uncommitted changes or be totally pristine.

### Trait: `EnsureGitClean`

- **Type Parameter**: `Error` - Defines custom errors encountered during validation.

- **Method**: `ensure_git_clean` - Examines the Git repository to ensure that the directory is free from uncommitted changes. It checks for the presence of a `.git` folder and uses the command `git status --porcelain` to detect any pending modifications.

### Design Details

- **Async Compatibility**: Utilizes async functions to run Git commands non-blockingly, offering smooth operation in asynchronous Rust programs.

- **Error Handling**: Provides structured error reporting including I/O related issues and informs if the Git working directory is not clean.

- **Flexible Implementation**: The trait can be implemented for various structures like `Workspace` or `CrateHandle`, expanding its usage across different scopes in a project.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
workspacer-git = "0.1.0"
```

## Usage

Implement the `EnsureGitClean` trait for your project structures to leverage its functionality.

```rust
#[async_trait]
impl<P, H> EnsureGitClean for Workspace<P, H> 
where
    P: From<PathBuf> + AsRef<Path> + Clone + Send + Sync,
    H: CrateHandleInterface<P> + Send + Sync {

    type Error = GitError;
    
    async fn ensure_git_clean(&self) -> Result<(), Self::Error> {
        // Implementation details
    }
}
```

## Licensing

[Included in full with the source code]

## Contributing

Contributions are welcomed. Please adhere to the [Code of Conduct] and submit any issues or pull requests to the project's repository.
