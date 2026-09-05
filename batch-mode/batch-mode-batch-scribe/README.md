# `batch-mode-batch-scribe`

Types and helpers for constructing and serializing language-model batch requests.
This is the successor to the former `gpt-batch-scribe` crate and is a member of the
[batch-mode workspace](../Cargo.toml).

## Current API

The main request type is `LanguageModelBatchAPIRequest`, with its derived
`LanguageModelBatchAPIRequestBuilder`. The [request implementation](src/language_model_batch_api_request.rs)
defines the current constructors and file-writing helper:

- `chat_completion_with_id(custom_id, system_msg, user_msg, model)` constructs a
  chat-completion request with a caller-supplied identifier.
- `new_basic(model, idx, system_message, user_message)` constructs a text request.
- `new_with_image(model, idx, system_message, user_message, image_b64)` constructs
  a request containing an image.
- `create_batch_input_file(requests, batch_input_filename)` writes batch request
  records as JSON lines and returns a `Result`.

The constructors take a `LanguageModelType`; the old `GptBatchAPIRequest` example
without a model argument does not describe this API. See the request implementation
for exact types and the [library entry point](src/lib.rs) for the related message,
request-body, identifier, batch-construction, and seed-manifest modules.

## Dependency maintenance

This crate inherits dependencies, including `async-openai`, from
[the workspace manifest](../Cargo.toml). Inspect the resolved dependency graph and
lockfile for that workspace when assessing transport dependencies such as Quinn.
The old `gpt-batch-scribe/Cargo.lock` is not the lockfile for this crate.

The former crate's [documentation](../../gpt-batch-scribe/README.md) is retained for
historical reference. Removing that retired crate's lockfile is repository cleanup,
not an upgrade of dependencies in existing builds or published releases.

## License

This project is licensed under the MIT License.
