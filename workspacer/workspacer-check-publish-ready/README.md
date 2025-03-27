# workspacer-check-publish-ready

`workspacer-check-publish-ready` is a Rust crate designed to streamline the verification of whether a Rust crate or workspace is prepared for publication on crates.io. It provides an asynchronous interface, leveraging the `ReadyForCargoPublish` trait to ensure comprehensiveness in publish readiness checks. 

## Features

- **Comprehensive Validations**: The crate facilitates a series of checks including validation of required fields in `Cargo.toml`, version validity, the presence of a README, and verification of the source directory structure.
- **Workspace Compatibility**: In addition to single crates, it can handle workspaces, ensuring that all contained crates meet the necessary criteria before publishing.
- **Error Compilation**: Collects and reports a detailed list of errors encountered during the publish readiness checks, aiding in efficient troubleshooting.

## Technical Details

- **Asynchronous Operations**: Built using Rust's async/await pattern, allowing non-blocking, concurrent checks across different components.
- **ReadyForCargoPublish Trait**: Core trait implemented for various components like `CargoToml`, `CrateHandle`, and `Workspace`, facilitating extensibility and reusability.

## Usage
Add `workspacer-check-publish-ready` to your `Cargo.toml` dependencies:

```toml
[dependencies]
workspacer-check-publish-ready = "0.1.0"
```

Implement the `ReadyForCargoPublish` trait for your components to carry out the necessary checks before publishing.

## License

`workspacer-check-publish-ready` is licensed under the MIT License. See [LICENSE](LICENSE) for more details.