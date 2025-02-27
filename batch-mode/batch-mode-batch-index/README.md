# batch-mode-batch-index

This crate provides utilities for handling batch indices in a batch processing environment. It defines the `BatchIndex` type, which can be either a `Usize` or a `Uuid`, and provides mechanisms to generate regular expressions (regex) for matching batch-related filenames based on these indices. The crate ensures the consistent handling of batch identifiers and file patterns throughout the batch processing lifecycle.

## Key Features
- **Batch Indexing**: Define and manage batch indices using either numeric (`Usize`) or UUID-based identifiers.
- **File Pattern Matching**: Generate regex patterns for matching batch file names (input, output, error, metadata) that incorporate the batch index.
- **Batch Index Type Conversion**: Easily convert between `BatchIndex` and `BatchIndexType`.
- **Edge Case Handling**: Robust handling of edge cases for both numeric and UUID-based batch indices.

## Usage

### Creating a Batch Index

```rust
use batch_mode_batch_index::{BatchIndex, BatchIndexType};

let batch_index = BatchIndex::Usize(4);
let regex = batch_index.file_pattern();
```

### Using Regex to Match Batch Filenames

```rust
assert!(regex.is_match("batch_input_4.jsonl"));
assert!(!regex.is_match("batch_input_invalid.jsonl"));
```

### Converting Between `BatchIndex` and `BatchIndexType`

```rust
let index_type = BatchIndexType::from(&batch_index);
```

### Error Handling

The crate defines errors for UUID parsing, ensuring proper validation and handling of batch index values.

## License
This crate is licensed under the MIT License. See LICENSE for details.
