# workspacer-syntax

The `workspacer-syntax` crate is a Rust library designed for parsing and generating signature strings for various abstract syntax tree (AST) nodes, such as functions, structs, enums, traits, type aliases, and macros. It facilitates both the creation of textual signatures and their rehydration from signature strings, thereby enabling syntactic reconstruction of Rust code components.

## Features

- **Signature Generation**: Create signature strings for AST nodes, including optional documentation comments.
- **Signature Options**: Tailor signature generation with options like full expansion of types and conditional inclusion of documentation lines.
- **Rehydration**: Convert signature strings back to AST nodes, facilitating round-trip conversions.
- **Public Visibility Checking**: Determine if a node represents a public interface element, such as a `pub fn` or `pub struct`.
- **Post-processing**: Correctly formats spacing around signature elements, enhancing legibility.

## Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
workspacer-syntax = "0.5.0"
```

### Example

Here is an example of generating a signature for a struct and retrieving its documentation:

```rust
use workspacer_syntax::{GenerateSignature, SignatureOptions};
use your_syntex_crate::ast;

let struct_ast: ast::Struct = /* Assume you have an AST node */;
let options = SignatureOptions::default();
let signature = struct_ast.generate_signature_with_opts(&options);

println!("Generated Signature:\n{}", signature);
```

### License

This project is licensed under either the MIT or Apache-2.0 license at your option.