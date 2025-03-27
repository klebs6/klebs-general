# workspacer-test-coverage

`workspacer-test-coverage` is a Rust library designed to facilitate the execution of tests and the collection of code coverage data within a cargo workspace. This crate leverages the `cargo-tarpaulin` tool to gather comprehensive coverage reports, either in JSON format or from plaintext summaries.

## Features

- **Asynchronous Test Execution**: Implements an asynchronous trait for running tests with coverage data collection.
- **Detailed Coverage Reporting**: Provides structures to represent coverage statistics including total coverage, lines covered, missed lines, and total lines.
- **Flexible Input Parsing**: Capable of parsing both JSON and plaintext output formats to generate coverage reports.
- **Error Handling**: Logs coverage parse errors and unexpected command failures, allowing for robust diagnostics.

## Installation

Add `workspacer-test-coverage` to your `Cargo.toml` dependencies:

```toml
[dependencies]
workspacer-test-coverage = "0.1.0"
```

## Usage

Implement the `RunTestsWithCoverage` trait in your workspace context to run tests and generate coverage reports:

```rust
#[async_trait]
impl<P, H: CrateHandleInterface<P>> RunTestsWithCoverage for Workspace<P,H> 
where for<'async_trait> P: From<PathBuf> + AsRef<Path> + Send + Sync + 'async_trait {
    // Implementation details
}
```

### Example

```rust
let workspace = Workspace::new("./my_workspace");
let coverage_report = workspace.run_tests_with_coverage().await?;
println!("Coverage: {:.2}%", coverage_report.total_coverage());
```

### Error Handling

The crate defines `TestCoverageError` to cover cases such as JSON parse errors, command execution failures, and coverage data anomalies.

## License

`workspacer-test-coverage` is licensed under the MIT License. See [LICENSE](./LICENSE) for more details.
