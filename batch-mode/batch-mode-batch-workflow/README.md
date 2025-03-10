# batch-mode-batch-workflow

This crate provides a specialized approach to batch-based GPT expansions and reconciling partial or incomplete batch states. It defines traits and workflows for:

- Reconciling uncompleted batches from prior runs,
- Computing new requests for language model expansions,
- Processing these requests in batches (locally or via remote APIs),
- Handling responses and results systematically.

## Key Traits

- **FinishProcessingUncompletedBatches** – Finalize partial data left from incomplete batch processing.
- **ComputeLanguageModelRequests** – Identify new items to process and generate requests to a language model API.
- **ProcessBatchRequests** – Handle chunked batch requests.
- **LanguageModelBatchWorkflow** – Integrates the above traits into a high-level, end-to-end batch-processing workflow.

## Getting Started

Add this crate to your `Cargo.toml`:

```toml
[dependencies]
batch-mode-batch-workflow = "*"
