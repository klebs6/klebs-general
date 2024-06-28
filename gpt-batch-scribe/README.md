# `gpt-token-scribe`

A Rust crate for handling batch API requests to GPT models. This crate provides structures and enumerations to construct, serialize, and deserialize JSON requests and handle errors appropriately.

## Features

- Define request structures with custom fields and methods.
- Serialize and deserialize using Serde.
- Handle various error types using a custom error tree.
- Support for different GPT models and API endpoints.
- Custom serialization for enums.

## Example Usage

### Basic Example

```rust
use gpt_token_scribe::{GptBatchAPIRequest, HttpMethod, GptApiUrl, GptRequestBody, GptMessage, GptModelType};

let request = GptBatchAPIRequest::new_basic(
    1,
    "You are a helpful assistant.",
    "Hello world!"
);

println!("{}", request);
```


## Structures and Enumerations

### `GptBatchAPIRequest`

Represents the complete request structure.

- custom_id: Identifier for the custom request.
- method: HTTP method used for the request.
- url: URL of the API endpoint.
- body: Body of the request.

Methods

- new_basic(idx: usize, system_message: &str, user_message: &str) -> Self: Creates a basic request.
- new_with_image(idx: usize, system_message: &str, user_message: &str, image_b64: &str) -> Self: Creates a request with an image.

### `GptApiUrl`

Enumeration of API URLs.

- ChatCompletions: API endpoint for chat completions.

### `GptMessage`

Represents individual message details in the request body.

- role: Role of the participant (system/user).
- content: Content of the message.

Methods

- system_message(msg: &str) -> Self: Creates a system message.
- user_message(msg: &str) -> Self: Creates a user message.
- user_message_with_image(msg: &str, image_b64: &str) -> Self: Creates a user message with an image.

### `GptModelType`

Supported model types.

- Gpt4o: Model type GPT-4o.
- Gpt4Turbo: Model type GPT-4 Turbo.

### `GptRequestBody`

Details of the API request body.

- model: Model used for the request.
- messages: Array of messages exchanged in the request.
- max_tokens: Maximum number of tokens to be used by the model.

Methods

- default_max_tokens() -> u32: Default maximum tokens.
- default_max_tokens_given_image(image_b64: &str) -> u32: Default maximum tokens when an image is included.
- new_basic(system_message: &str, user_message: &str) -> Self: Creates a basic request body.
- new_with_image(system_message: &str, user_message: &str, image_b64: &str) -> Self: Creates a request body with an image.

### `HttpMethod`

Enumeration of possible HTTP methods.

- Get: HTTP GET method.
- Post: HTTP POST method.

Error Handling

### `ParseTokenDescriptionLineError`

Enumeration for parsing token description line errors.

- MissingToken
- MissingDescription

### `TokenizerError`

Enumeration for tokenizer errors.

- TokenizerError(String)

### `GptBatchCreationError`

Enumeration for batch creation errors.

- OpenAIError(OpenAIError)
- IOError(std::io::Error)
- TokenizerError(TokenizerError)
- ParseTokenDescriptionLineError(ParseTokenDescriptionLineError)
- SerdeJsonError(serde_json::Error)

## License

This project is licensed under the MIT License.
