# batch-mode-batch-schema

The `batch-mode-batch-schema` crate provides the data structures and schemas necessary for representing and handling batch-related data in batch processing workflows. It includes structures for batch choices, error responses, token details, and other key elements that are essential for processing batches in a consistent way. The crate also offers the ability to deserialize JSON responses, ensuring the integrity of the batch data and handling any errors that may arise.

## Key Features
- **Batch Choices**: Represents the choices made during batch processing, including the batch index, message, log probabilities, and finish reason.
- **Message Role Management**: Handles different message roles (assistant, user, system), with support for unknown roles.
- **Error Handling**: Includes error structures that manage batch errors and their responses, allowing for consistent handling and reporting.
- **Batch Response Handling**: Provides structures for processing batch success and error responses, including the ability to retrieve details about the response such as choices, model, usage, and more.
- **Batch Token Details**: Tracks token usage across different stages of batch processing, including cached, audio, and reasoning tokens.
- **Deserialization with Validation**: Automatically handles JSON deserialization with strict validation to ensure the integrity of data, rejecting invalid fields or missing required fields.

## Usage

### Creating and Using BatchChoice

```rust
use batch_mode_batch_schema::{BatchChoice, FinishReason, BatchMessage};
use serde_json::json;

let json = r#"{
    "index": 0,
    "message": {
        "role": "assistant",
        "content": "This is the assistant's response.",
        "refusal": null
    },
    "logprobs": null,
    "finish_reason": "stop"
}"#;

let choice: BatchChoice = serde_json::from_str(json).unwrap();
println!("Batch Choice: {:?}", choice);
```

### Handling Message Roles

```rust
use batch_mode_batch_schema::MessageRole;
use serde_json::json;

let json = r#""assistant""#;
let role: MessageRole = serde_json::from_str(json).unwrap();
println!("Message Role: {:?}", role);
```

### Batch Error Handling

```rust
use batch_mode_batch_schema::{BatchErrorResponseBody, BatchErrorDetails};

let error_json = r#"{
    "error": {
        "message": "Invalid API key",
        "type": "authentication_error",
        "param": null,
        "code": "invalid_api_key"
    }
}"#;

let error_body: BatchErrorResponseBody = serde_json::from_str(error_json).unwrap();
println!("Error: {:?}", error_body.error());
```

### Token Details

```rust
use batch_mode_batch_schema::BatchTokenDetails;
use serde_json::json;

let token_json = r#"{
    "cached_tokens": 100,
    "audio_tokens": 50,
    "reasoning_tokens": 25,
    "accepted_prediction_tokens": 20,
    "rejected_prediction_tokens": 5
}"#;

let token_details: BatchTokenDetails = serde_json::from_str(token_json).unwrap();
println!("Total tokens: {}", token_details.total_tokens());
```

## Error Handling

This crate utilizes the `error-tree` library for structured error handling, including errors for invalid content, missing required fields, and deserialization issues. Some of the key error types are:
- `ContentParseError`: Indicates errors while parsing batch content.
- `BatchReconciliationError`: Indicates errors in the reconciliation process.
- `JsonParseError`: Indicates issues during JSON parsing.

## License
This crate is licensed under the MIT License. See LICENSE for details.
