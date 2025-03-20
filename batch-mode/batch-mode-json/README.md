# batch-mode-json

The `batch-mode-json` crate provides utilities for extracting JSON content from text blocks, writing JSON data to files, and handling JSON parsing errors. This crate is useful for working with batch processing data that involves JSON-formatted content.

## Key Features
- **JSON Extraction**: Extracts JSON content from a string, especially when it’s wrapped in specific markers such as ` ```json ... ``` `.
- **Asynchronous File Writing**: Provides asynchronous functionality for writing serialized JSON content to files.
- **Error Handling**: Includes error handling for common JSON parsing issues, such as invalid JSON format or errors during file I/O operations.

## Modules and Functions

### Extract JSON from a Possible Backticks Block
The crate offers a function `extract_json_from_possible_backticks_block` that extracts JSON content from a string, handling surrounding backtick markers (` ```json ... ``` `) and whitespace.

```rust
pub fn extract_json_from_possible_backticks_block(content: &str) -> &str;
```

### Write JSON to File
The `write_to_file` function asynchronously writes a given JSON string to a file. It handles file creation and ensures all data is flushed to disk.

```rust
pub async fn write_to_file(target_path: impl AsRef<Path>, serialized_json: &str) -> Result<(), io::Error>;
```

### Error Handling
The crate also provides an error enum `JsonParseError` to capture various issues during JSON parsing and file handling.

```rust
error_tree!{
    pub enum JsonParseError {
        JsonRepairError(JsonRepairError),
        InvalidJson,
        SerdeError(serde_json::Error),
        IoError(std::io::Error),
    }
}
```

## Usage

### Extract JSON Content
You can extract JSON content from a string that might be surrounded by backticks using the `extract_json_from_possible_backticks_block` function:

```rust
let content = "```json\n{\"key\": \"value\"}\n```";
let json_content = extract_json_from_possible_backticks_block(content);
pretty_assert_eq!(json_content, "{\"key\": \"value\"}");
```

### Write JSON to a File
To write JSON data to a file asynchronously, use the `write_to_file` function:

```rust
use batch_mode_json::write_to_file;
let json_data = r#"{"key": "value"}"#;
write_to_file("output.json", json_data).await.unwrap();
```

## License
This crate is licensed under the MIT License. See LICENSE for details.
