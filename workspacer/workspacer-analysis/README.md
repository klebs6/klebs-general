# workspacer-analysis

`workspacer-analysis` is a Rust library designed for exhaustive statistical analysis of Rust workspaces. It efficiently processes large codebases, providing detailed insights into source and test files across multiple crates using asynchronous computations.

## Features

- **Asynchronous Analysis**: Leveraging Rust's `async` capabilities, the library conducts analyses without blocking, enhancing performance in typical development environments.
- **Comprehensive Metrics**: Provides metrics such as total file size, lines of code, count of source and test files, and more.
- **Scalable**: Can handle multiple crates efficiently, making it suitable for large projects.

## Use Cases

Ideal for developers and teams needing to analyze codebase metrics simultaneously across distinct crates in a Rust workspace. It aids in pinpointing inefficiencies and optimizing code organization.

## Example Usage

```rust
use workspacer_analysis::{Analyze, CrateAnalysis, Workspace};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workspace = Workspace::new("path/to/workspace")?;
    let analysis = workspace.analyze().await?;
    println!("Total file size: {} bytes", analysis.total_file_size());
    Ok(())
}
```

## Requirements

- Rust 2024 edition for leveraging the latest language features.
- Tokio for asynchronous execution.

## Contributions

Contributions are welcome! Please feel free to submit a pull request or open an issue.

## License

This project is licensed under the MIT License.