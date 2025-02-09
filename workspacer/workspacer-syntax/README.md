# workspacer-syntax

`workspacer-syntax` is a utility crate in the Workspacer ecosystem that provides tools for processing and analyzing Rust source code at the syntax (AST) level. It enables you to:

- **Extract Documentation:**  
  Use the `extract_docs` function to collect documentation comments (e.g. `///` and `/** ... */`) from a syntax node.

- **Check Public Visibility:**  
  Determine if an AST node is public via the `is_node_public` function by examining its visibility modifiers and attributes.

- **Generate Signatures:**  
  Implement the `GenerateSignature` trait for common Rust items (functions, structs, enums, traits, type aliases, and macro rules) to produce standardized signature strings with optional documentation.

This crate is a core component for building automated interface extraction, documentation generation, and analysis tools for Rust projects.

## Features

- **Documentation Extraction:**  
  The `extract_docs` function scans an AST node’s tokens and collects its doc comments into a single string.

- **Public Visibility Check:**  
  The `is_node_public` function checks if a syntax node is public by inspecting its children and attributes (including special handling for macros).

- **Signature Generation:**  
  The `GenerateSignature` trait is implemented for various AST node types (e.g. `ast::Fn`, `ast::Struct`, `ast::Trait`, etc.) to generate human-readable signatures.

## Installation

Add `workspacer-syntax` to your `Cargo.toml` dependencies:

```toml
[dependencies]
workspacer-syntax = "0.1.0"
```

## Usage

Below is an example that demonstrates how to use workspacer-syntax to extract documentation and generate a signature from an AST node:

```rust
use workspacer_syntax::{extract_docs, is_node_public, GenerateSignature};
use ra_ap_syntax::{SyntaxNode, SyntaxKind};
use ra_ap_syntax::ast;

fn process_node(node: &SyntaxNode) {
    if is_node_public(node) {
        if let Some(docs) = extract_docs(node) {
            println!("Documentation:\n{}", docs);
        }
        // If the node represents a function, generate its signature.
        if let Some(func) = ast::Fn::cast(node.clone()) {
            let signature = func.generate_signature(None);
            println!("Signature: {}", signature);
        }
    }
}
```

This example assumes you have a valid `SyntaxNode` from the RA (Rust Analyzer) parser, and it demonstrates how to check for public visibility, extract documentation, and generate a signature for a function.

## License

This project is dual-licensed under either the [MIT license](LICENSE-MIT) or the [Apache License, Version 2.0](LICENSE-APACHE), at your option.

## Repository

For more information and to contribute, please visit the [GitHub repository](https://github.com/klebs6/klebs-general).
