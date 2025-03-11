# language-model-batch-workflow-json-output-derive

A **proc-macro crate** that allows you to derive a trait, `AiJsonTemplate`, on any plain-old Rust struct to produce a **JSON “schema”** or “template” describing that struct’s fields, doc comments, and nested structures. This crate enforces certain field types—such as `String`, `Option<String>`, `Vec<String>`, or a recursively nested type also deriving `AiJsonTemplate`—making it useful for guiding AI models (like GPT) to output data matching a specific format.

## Features

1. **`AiJsonTemplate` Trait**  
   - Automatically implemented via `#[derive(AiJsonTemplate)]`.  
   - Provides a single method, `to_template()`, returning a `serde_json::Value` describing each field’s type and any doc comments.

2. **Doc Comments to Instructions**  
   - Rust doc comments (`///`) on structs and fields become embedded in the JSON output, helping you generate AI instructions or clarifications.

3. **Nested Struct Support**  
   - If a field’s type also derives `AiJsonTemplate`, the macro includes a `"nested_template"` object in the JSON.

4. **Controlled Field Types**  
   - By default, only `String`, `Option<String>`, `Vec<String>`, or nested `AiJsonTemplate` types are allowed. The macro fails if it encounters other field types, ensuring consistent data structures for AI outputs.

5. **Seamless with Serde**  
   - You can also derive `Serialize` and `Deserialize` on the same struct. The macro doesn’t interfere with normal Rust <-> JSON round-trip usage.

## Example

```rust
use language_model_batch_workflow_json_output_derive::AiJsonTemplate;
use serde::{Serialize, Deserialize};

#[derive(AiJsonTemplate, Serialize, Deserialize)]
/// My top-level config struct
pub struct TopLevelConfig {
    /// Plain string field, always required
    title: String,

    /// Optional field for additional notes
    notes: Option<String>,

    /// Another struct, nested
    nested: SubConfig,
}

#[derive(AiJsonTemplate, Serialize, Deserialize)]
/// A nested struct, also with doc comments
pub struct SubConfig {
    /// A short summary
    summary: String,

    /// Multiple tags
    tags: Vec<String>,
}

fn main() {
    // Generate the template describing fields + doc comments
    let schema = TopLevelConfig::to_template();
    println!("JSON template:\n{}",
        serde_json::to_string_pretty(&schema).unwrap()
    );
}
```

**Output** might look like:

```json
{
  "struct_name": "TopLevelConfig",
  "struct_docs": "My top-level config struct",
  "fields": {
    "title": {
      "type": "string",
      "docs": "Plain string field, always required",
      "required": true
    },
    "notes": {
      "type": "string",
      "docs": "Optional field for additional notes",
      "required": false
    },
    "nested": {
      "type": "nested_struct",
      "docs": "Another struct, nested",
      "required": true,
      "nested_template": {
        "struct_name": "SubConfig",
        "struct_docs": "A nested struct, also with doc comments",
        "fields": {
          "summary": {
            "type": "string",
            "docs": "A short summary",
            "required": true
          },
          "tags": {
            "type": "array_of_strings",
            "docs": "Multiple tags",
            "required": true
          }
        }
      }
    }
  }
}
```

## Installation

In your `Cargo.toml`, add:

```toml
[dependencies]
language-model-batch-workflow-json-output-derive = "0.1"
serde                                            = "1.0"
serde_json                                       = "1.0"
```

> **Note**: Because it’s a proc-macro crate, ensure you have Rust 2021 edition or later. You’ll also need `syn`, `quote`, and `proc-macro2` available internally.

## Usage

1. **Import the macro**: 
   ```rust
   use language_model_batch_workflow_json_output_derive::AiJsonTemplate;
   ```
2. **Annotate structs** with `#[derive(AiJsonTemplate, Serialize, Deserialize)]`.
3. **Generate JSON**: call `YourStruct::to_template()` to retrieve a structured schema of doc comments and types.  
4. **Feed** that JSON schema to an AI model to guide output format, or use it as you see fit.

## Testing

We provide a robust integration test suite in the `tests/` folder, covering:

- **Simple** usage with required and optional fields.
- **Nested** structs (multi-level).
- **Doc comments** verification.
- **Round-trip** checks ensuring normal Serde usage is unaffected.

You can also add [trybuild] tests to confirm that the macro fails gracefully when encountering unsupported field types or missing `Serialize`/`Deserialize` derivations.
