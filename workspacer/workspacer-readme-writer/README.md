# workspacer-readme-writer

## Overview

`workspacer-readme-writer` is a Rust crate designed to facilitate the generation and updating of `README.md` and `Cargo.toml` files within crate or workspace environments. This is achieved by leveraging AI-driven expansions, making it suitable for projects that require automation in documentation management.

## Key Features
- **AI-Powered Content Generation**: Utilizes language models to generate informative and detailed `README.md` content tailored for intelligent and discerning audiences.
- **Automatic Cargo.toml Update**: Seamlessly modifies the description, keywords, and categories within the `Cargo.toml` based on AI outputs.
- **Modular and Extensible Interface**: Implements traits such as `ApplyAiReadmeOutput` for README and Cargo.toml modifications, ensuring extensibility.
- **Path and Handle Management**: Stores and retrieves crate handles efficiently to maintain associations with workspace directories.

## Architecture
The crate employs a variety of async traits and mechanisms:
- `ReadmeWritingCrateHandle<P>`: Interface for crate handle operations.
- `ApplyAiReadmeOutput`: Asynchronous methods for updating README and Cargo.toml.
- `AiReadmeWriterRequest`: Holds necessary request information and metadata, enabling precise expansions and updates.
- `AiReadmeWriter`: Central coordinator for generating and applying AI-driven README and Cargo.toml modifications.

## Getting Started
Integrate `workspacer-readme-writer` into your project by including it in your dependencies within `Cargo.toml`. Create an `AiReadmeWriter` instance and invoke the `execute_ai_readme_writer_requests` method with appropriately crafted requests to automate your documentation workflow.

## Installation
To add `workspacer-readme-writer` to your project, update your `Cargo.toml` as follows:

```toml
[dependencies]
workspacer-readme-writer = "0.1.0"
```

## Usage Example
```rust
use workspacer_readme_writer::{AiReadmeWriter, AiReadmeWriterRequest};
use std::sync::Arc;
use async_std::path::PathBuf;

#[tokio::main]
async fn main() {
    let writer = AiReadmeWriter::default().await.unwrap();
    let requests = vec![
        AiReadmeWriterRequest::<PathBuf>::async_try_from(/* crate_handle */).await.unwrap()
    ];
    writer.execute_ai_readme_writer_requests(&requests).await.unwrap();
}
```

## Contributing
Contributions to enhance this crate are welcome. Please follow [Rust's standards](https://www.rust-lang.org/policies/code-of-conduct) for code formatting and ensure thorough testing.
