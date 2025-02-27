# batch-mode-batch-metadata

The `batch-mode-batch-metadata` crate provides utilities for managing and persisting metadata associated with batch processing workflows. This includes storing and loading metadata related to batches, such as input, output, and error file IDs. It leverages the `SaveToFile` and `LoadFromFile` traits to persist metadata to disk and retrieve it for further processing.

## Key Features
- **Batch Metadata Management**: Manage metadata for batches, including batch ID, input file ID, output file ID, and error file ID.
- **Serialization**: Serialize and deserialize batch metadata to and from JSON format, with robust error handling.
- **File Persistence**: Easily save and load batch metadata from files using the `SaveToFile` and `LoadFromFile` traits.
- **Error Handling**: Detailed error handling for missing file IDs, serialization issues, and IO errors.

## Usage

### Creating Batch Metadata

```rust
use batch_mode_batch_metadata::BatchMetadata;

let metadata = BatchMetadata::with_input_id_and_batch_id("input_id", "batch_id");
```

### Setting and Retrieving File IDs

```rust
metadata.set_output_file_id(Some("output_file_id".to_string()));
metadata.set_error_file_id(Some("error_file_id".to_string()));

let output_file_id = metadata.output_file_id().unwrap();
let error_file_id = metadata.error_file_id().unwrap();
```

### Saving Metadata to File

```rust
metadata.save_to_file("metadata_file.json").await?;
```

### Loading Metadata from File

```rust
let loaded_metadata = BatchMetadata::load_from_file("metadata_file.json").await?;
```

### Error Handling

This crate uses `error-tree` for managing errors such as missing file IDs and IO or serialization errors.

## Error Types

- `BatchMetadataError::MissingOutputFileId`: When the output file ID is missing.
- `BatchMetadataError::MissingErrorFileId`: When the error file ID is missing.
- `BatchMetadataError::SerializationError`: When serialization fails.
- `BatchMetadataError::IoError`: When an IO error occurs during file reading/writing.

## Requirements
- Requires `serde`, `serde_json`, and `async_trait` for serialization and file operations.

## License
This crate is licensed under the MIT License. See LICENSE for details.
