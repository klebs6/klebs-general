# camel-case-token-with-comment

The `camel-case-token-with-comment` crate provides functionality for handling and parsing tokens in camel case format with optional comments. It includes utilities for processing token files, extracting relevant token fields from JSON, and serializing tokens to and from strings.

## Key Features
- **CamelCaseTokenWithComment**: A structure that encapsulates a token in camel case format with an optional comment.
- **File Parsing**: Functions to parse token files that contain tokens with optional comments.
- **JSON Field Extraction**: Extracts the `token_name` field from JSON data.

## Modules and Functions

### CamelCaseTokenWithComment Structure
The `CamelCaseTokenWithComment` struct holds a camel case token with an optional comment. This structure supports various utilities for managing the data.

```rust
pub struct CamelCaseTokenWithComment {
    data: String,
    comment: Option<String>,
}
```

### Parsing Token Files
The crate includes the function `parse_token_file`, which asynchronously reads a file and parses each line into `CamelCaseTokenWithComment`.

```rust
pub async fn parse_token_file(filename: &str) -> Result<Vec<CamelCaseTokenWithComment>, TokenParseError>;
```

### Extract Token Name Field
A utility function `extract_token_name_field` extracts the "token_name" field from JSON data.

```rust
pub fn extract_token_name_field(json: &serde_json::Value) -> Result<&str, TokenParseError>;
```

## Error Handling
This crate uses custom error types to handle parsing and I/O errors:

```rust
error_tree! {
    pub enum TokenParseError {
        InvalidTokenName,
        InvalidTokenLine(String),
        MissingTokenNameField,

        #[cmp_neq]
        IoError(std::io::Error),
    }
}
```

## Usage

### Parse a Token File
To parse a file containing tokens with optional comments:

```rust
let tokens = parse_token_file("path_to_file.txt").await?;
```

### Extract Token Name from JSON
To extract the `token_name` field from a JSON object:

```rust
let json = serde_json::json!({ "token_name": "example_token" });
let token_name = extract_token_name_field(&json)?;
```

## License
This crate is licensed under the MIT License. See LICENSE for details.
