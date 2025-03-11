# language-model-batch-workflow-derive

This crate provides a procedural macro `#[derive(LanguageModelBatchWorkflow)]` for structures that manage automated batch tasks involving language model requests. By annotating certain fields with attributes like `#[batch_client]` and `#[batch_workspace]`, you can generate consistent, boilerplate-free implementations of core traits from the [batch-mode-batch-workflow](https://github.com/klebs6/klebs-general) ecosystem.

## Overview

When deriving `LanguageModelBatchWorkflow`, the macro enforces the presence of several annotated fields:

- **`#[batch_client]`** – Must be an `Arc<OpenAIClientHandle>` or `Arc<dyn LanguageModelClientInterface<E>>`, where `E` is your custom error type or `OpenAIClientError`.
- **`#[batch_workspace]`** – Must be an `Arc<BatchWorkspace>` or `Arc<dyn FullBatchWorkspaceInterface<BatchWorkspaceError>>`.
- **`#[expected_content_type]`** – Must be an `ExpectedContentType`.
- **`#[model_type]`** – Must be a `LanguageModelType`.
- **`#[batch_error_type(...)]`** – Declares the user’s chosen custom error type (e.g., `MyErr`) at the struct level.

Additionally, you may supply optional attributes to customize output/error processing:

- **`#[custom_process_batch_output_fn]`** – A `BatchWorkflowProcessOutputFileFn` that defines how to handle successful batch results.
- **`#[custom_process_batch_error_fn]`** – A `BatchWorkflowProcessErrorFileFn` that defines how to handle batch errors.

## Key Traits Implemented

1. **`FinishProcessingUncompletedBatches`**  
   Provides logic to finalize any partially processed batch work using the annotated workspace and client fields.

2. **`ProcessBatchRequests`**  
   Defines how to dispatch new requests and handle output/error files, with optional custom overrides for post-processing.

3. **`LanguageModelBatchWorkflow<Error>`**  
   Supplies the `plant_seed_and_wait` method, which internally invokes your primary batch workflow logic asynchronously.

4. **`ComputeLanguageModelRequests`** (to be implemented by you)  
   Although not generated automatically, you provide the method that forms your language model queries (`compute_language_model_requests`). The macro integrates this method into the larger workflow.

5. **`Send` and `Sync`**  
   The derived struct is declared thread-safe to allow concurrency in batch-based tasks.

## Example

```rust
use language_model_batch_workflow_derive::LanguageModelBatchWorkflow;
use batch_mode_batch_workflow::*;
use batch_mode_3p::*;
use std::sync::Arc;

/// Define your own error type (must implement `From` conversions for relevant errors).
#[derive(Debug)]
pub struct MyErr;

// Satisfy all required conversions, for example:
impl From<OpenAIClientError> for MyErr {
    fn from(_err: OpenAIClientError) -> Self {
        MyErr
    }
}

// ...

#[derive(LanguageModelBatchWorkflow)]
#[batch_error_type(MyErr)]
struct MyBatchWorker {
    /// Connects to the language model (e.g., OpenAI).
    #[batch_client]
    client: Arc<dyn LanguageModelClientInterface<MyErr>>,

    /// Manages directories and file I/O for batch stages.
    #[batch_workspace]
    workspace: Arc<BatchWorkspace>,

    /// Specifies the content type (e.g., JSON).
    #[expected_content_type]
    ect: ExpectedContentType,

    /// Selects which model to use.
    #[model_type]
    mt: LanguageModelType,

    /// Optionally define custom post-processing logic.
    #[custom_process_batch_output_fn]
    pbo: BatchWorkflowProcessOutputFileFn,

    #[custom_process_batch_error_fn]
    pbe: BatchWorkflowProcessErrorFileFn,
}

impl ComputeLanguageModelRequests for MyBatchWorker {
    type Seed = ();

    fn compute_language_model_requests(
        &self,
        _model: &LanguageModelType,
        _input_tokens: &[Self::Seed]
    ) -> Vec<LanguageModelBatchAPIRequest> {
        // Your logic here
        vec![]
    }
}
```

In this example, the macro checks that each annotated field is of a valid type, then automatically implements batch workflow traits. You only need to supply your custom logic for building language model requests.

## Trybuild Tests

This crate uses [trybuild](https://github.com/dtolnay/trybuild) to verify correctness by compiling pass/fail examples:

- **`fail_missing_batch_client.rs`**  
  Ensures a compile error occurs if `#[batch_client]` is absent.
- **`fail_missing_batch_workspace.rs`**  
  Checks that `#[batch_workspace]` is mandatory.
- **`fail_missing_error_type.rs`**  
  Ensures the macro fails when `#[batch_error_type(...)]` is not provided.
- **`pass_valid_struct.rs`**  
  Demonstrates a proper struct that compiles without error.
