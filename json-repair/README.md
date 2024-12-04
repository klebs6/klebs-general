
# JSON Repair for Rust

A Rust library for repairing malformed JSON strings and parsing them into valid JSON values.

## Overview

`json-repair` is a Rust crate that provides utilities to repair and parse malformed JSON strings. It handles common JSON syntax errors such as:

- Unclosed braces or brackets
- Missing commas between items
- Incorrect use of single quotes instead of double quotes
- Truncated strings or booleans
- Control characters within strings
- Mismatched brackets or quotes

This library is particularly useful when dealing with JSON data from unreliable sources where the JSON may not be well-formed.

## Features

- **Robust Repair Mechanisms**: Attempts to fix a variety of common JSON syntax errors.
- **Detailed Error Handling**: Provides specific error types to help identify why a repair might have failed.
- **Extensive Testing**: Comes with a comprehensive suite of unit tests covering various malformed JSON scenarios.

## Installation

Add `json-repair` to your `Cargo.toml` dependencies:

```toml
[dependencies]
json-repair = "0.1.0"
```

Then, include it in your Rust code:

```rust
use json_repair::repair_json_string;
```

## Usage

Here's how you can use `json-repair` to repair a malformed JSON string:

```rust
use json_repair::repair_json_string;
use serde_json::Value;

fn main() {
    let malformed_json = r#"{
        'key1': 'value1',
        'key2': "value2",
        "key3": 'value3',
        "text": "Don't stop believing",
        'another_text': 'It\'s a kind of magic',
        "nested": {
            'inner_key': 'inner_value'
        }
    }"#;

    match repair_json_string(malformed_json) {
        Ok(json_value) => {
            println!("Repaired JSON: {}", json_value);
        },
        Err(e) => {
            eprintln!("Failed to repair JSON: {}", e);
        }
    }
}
```

**Output:**

```json
{
  "key1": "value1",
  "key2": "value2",
  "key3": "value3",
  "text": "Don't stop believing",
  "another_text": "It's a kind of magic",
  "nested": {
    "inner_key": "inner_value"
  }
}
```

## API

### `repair_json_string`

```rust
fn repair_json_string(input: &str) -> Result<Value, JsonRepairError>
```

Attempts to repair the given JSON string and parse it into a `serde_json::Value`.

#### Parameters

- `input`: The malformed JSON string to repair.

#### Returns

- `Ok(Value)`: The repaired JSON parsed into a `serde_json::Value`.
- `Err(JsonRepairError)`: An error indicating why the repair failed.

### Error Types

The `JsonRepairError` enum provides detailed error variants for different failure cases:

- `FailedToParseRepairedJson`: The repaired string could not be parsed into JSON.
- `Unrepairable(String)`: The input string is unrepairable, with details provided.
- `AllAttemptedRepairsFailed`: All repair attempts have failed.
- `CouldNotConvertTheOutputOfDuplicateQuoteRemovalToJson`: Failed to parse JSON after attempting to remove duplicate quotes.
- `SerdeParseError`: An error from `serde_json` during parsing.

## Supported Repairs

The library attempts the following repairs:

1. **Accidental Single Quotes**: Converts single-quoted strings to double-quoted strings.
2. **Missing Commas**: Inserts missing commas between JSON elements.
3. **Mismatched Brackets**: Fixes mismatched or missing brackets and braces.
4. **Truncated Booleans**: Completes partially written `true` or `false` literals.
5. **Control Characters**: Removes control characters that are invalid in JSON strings.
6. **Duplicate Quotes**: Removes duplicate quotes in strings.
7. **Unclosed Structures**: Closes any unclosed braces, brackets, or strings.
8. **Mismatched Quotes**: Fixes mismatched or missing quotes in strings.
9. **Unexpected EOF**: Handles unexpected end-of-file errors by closing open structures.

## Examples

### Repairing Missing Commas and Single Quotes

```rust
use json_repair::repair_json_string;
use serde_json::json;

let malformed_json = r#"{
    "key": [
        "value1",
        "value2"
        "value3", // Missing comma
        'value4', // Single quotes instead of double quotes
        "value5"
    ]
}"#;

let repaired = repair_json_string(malformed_json).unwrap();

assert_eq!(repaired, json!({
    "key": [
        "value1",
        "value2",
        "value3",
        "value4",
        "value5"
    ]
}));
```

### Handling Truncated JSON

```rust
use json_repair::repair_json_string;
use serde_json::json;

let truncated_json = r#"{
    "list": [1, 2, 3
"#; // Missing closing brackets

let repaired = repair_json_string(truncated_json).unwrap();

assert_eq!(repaired, json!({
    "list": [1, 2, 3]
}));
```

### Fixing Mismatched Brackets

```rust
use json_repair::repair_json_string;
use serde_json::json;

let malformed_json = r#"{
    "data": [
        {"id": 1, "value": "A"},
        {"id": 2, "value": "B"],
    "status": "ok"
}"#; // Mismatched brackets

let repaired = repair_json_string(malformed_json).unwrap();

assert_eq!(repaired, json!({
    "data": [
        {"id": 1, "value": "A"},
        {"id": 2, "value": "B"}
    ],
    "status": "ok"
}));
```

## Limitations

While `json-repair` can handle many common JSON syntax errors, it cannot repair all possible malformed JSON inputs. If the input is too corrupted or ambiguous, the repair may fail, and an appropriate error will be returned.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request if you have suggestions or improvements.

## License

This project is licensed under the [MIT License](LICENSE).

---

**Note:** This crate relies on the `serde_json` library for JSON parsing and the `json5` library for initial parsing attempts. It also uses the `regex` crate for pattern matching during the repair process.
