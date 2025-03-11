# language-model-token-expander

This crate provides a high-level, batch-oriented token expansion system. By integrating with the [`batch-mode-batch-workflow`](https://github.com/klebs6/klebs-general) tooling and a language model client (e.g., OpenAI), it streamlines processing tokens in batch, generating requests, and reconciling outputs.

## Features

- **`LanguageModelTokenExpander` Struct**  
  - Uses `#[derive(LanguageModelBatchWorkflow)]` to implement a comprehensive batch workflow.  
  - Leverages the `CreateLanguageModelRequestsAtAgentCoordinate` trait to define how requests are formed.
  - Manages workspace, client handles, and metadata for robust batch processing.

- **Modular Error Type**  
  - `TokenExpanderError` consolidates a range of possible error variants (e.g., file I/O, reconciliation errors) into one convenient enum.

- **`ComputeLanguageModelRequests` Integration**  
  - Automatically extracts unseen tokens from the workspace and creates language model requests in an extensible, trait-driven manner.

## Usage

```rust
use language_model_token_expander::*;
use batch_mode_token_expander::CreateLanguageModelRequestsAtAgentCoordinate;
use std::sync::Arc;
use agent_coordinate::AgentCoordinate;

#[tokio::main]
async fn main() -> Result<(), TokenExpanderError> {
    // Provide an implementation for CreateLanguageModelRequestsAtAgentCoordinate
    struct MyRequestCreator;
    impl CreateLanguageModelRequestsAtAgentCoordinate for MyRequestCreator {
        fn create_language_model_requests_at_agent_coordinate<X: IntoLanguageModelQueryString>(
            &self,
            model: &LanguageModelType,
            coord: &AgentCoordinate,
            inputs: &[X],
        ) -> Vec<LanguageModelBatchAPIRequest> {
            // custom request creation
            vec![]
        }
    }

    let my_expander = LanguageModelTokenExpander::new(
        "/path/to/batch_workspace",
        Arc::new(MyRequestCreator),
        AgentCoordinate::default(),
        LanguageModelType::default(),
        ExpectedContentType::Json,
    ).await?;

    // Provide seeds (tokens) to expand
    let seeds = vec![];
    my_expander.plant_seed_and_wait(&seeds).await?;

    Ok(())
}
```

In this example, `LanguageModelTokenExpander` automatically handles workspace management and organizes the batch flow from seed input to final JSON output. You need only define how to convert your tokens into `LanguageModelBatchAPIRequest` structures.
