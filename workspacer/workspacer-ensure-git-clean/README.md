# workspacer-ensure-git-clean

`workspacer-ensure-git-clean` is a crate intended for Rust engineers who require certainty in the cleanliness of a Git workspace before performing operations that should not proceed in a dirty state, such as deployment or integration tasks.

## Overview

The crate provides a utility to assert and enforce that a Git workspace is devoid of uncommitted changes or untracked files. This ensures the integrity and consistency of operations that are dependent on a stable code base.

## Features
- **Clean State Check**: Evaluates whether the current workspace is clean by examining the `git status`.
- **Error Feedback**: Provides informative error messages if the workspace is not clean.
- **Integration Simplicity**: Designed to be readily embedded into existing Rust workflows, particularly in CI/CD pipelines.

## Usage
To incorporate it into your Rust project, add it to your `Cargo.toml`:

```toml
[dependencies]
workspacer-ensure-git-clean = "0.1.0"
```

Then, in your Rust source file:

```rust
use workspacer_ensure_git_clean::ensure_clean;

fn main() {
    ensure_clean().expect("Workspace must be clean before proceeding.");
}
```

## Technical Details
This crate makes use of the Rust 2024 edition and assumes familiarity with Git operations at a technical level. It leverages underlying system Git commands to ascertain state, requiring that Git is installed and available in the environment where the Rust code is executed.

## Contribution
Contributions via the standard GitHub pull request workflow are welcomed, especially regarding expanded platform support and enhancing feedback clarity in diverse environments.
