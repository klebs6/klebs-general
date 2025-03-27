# workspacer-cratesio-mock

`workspacer-cratesio-mock` is a mock in-memory interface designed for simulating the publishing and storage operations typically executed on crates.io. This crate is particularly useful in scenarios necessitating the testing of client-crate interactions without direct dependence on the actual crates.io infrastructure.

## Overview

The core of the crate is an in-memory `MockCratesDb`, which acts as a temporary storage for published crates. This crate utilizes Rust's advanced asynchronous features and concurrency primitives like `Arc` and `AsyncMutex` to efficiently manage concurrent state changes in simulated publishing operations.

### Key Components

- **MockCratesDb**: A data structure implementing a `HashMap` to represent the published crates database. This ensures efficient lookup and storage, simulating crates.io’s backend structure.

- **StoredCrate**: Represents a crate with essential fields like `name`, `vers` (version), and an optional `description`, vital for crate metadata.

- **PublishOkResponse & PublishErrResponse**: Define structured responses for publishing outcomes, encapsulating success or detailed error information.

- **API Endpoints**: Provides RESTful routes using Rocket framework for handling crate publishing requests, ensuring robust validation and feedback mechanisms.

## Features
- **Concurrency Support**: Leverages `Arc` and `AsyncMutex` for concurrent access.
- **Error Handling**: Comprehensive error reporting and management system.
- **Extensible Design**: Additional routes and handling can be integrated easily, making it customizable for various testing requirements.

## Usage

This crate is targeted toward developers looking to test crate publication processes. It can be integrated in CI/CD workflows to validate crate metadata handling before actual publication.

### Example

```rust
use workspacer_cratesio_mock::{MockCratesDb, AppState};
use rocket::State;

#[tokio::main]
async fn main() {
    let mock_db = MockCratesDb::default();
    let app_state = AppState { db: Arc::new(AsyncMutex::new(mock_db)) };
    
    rocket::build()
        .manage(app_state)
        .mount("/api/v1/crates", routes![publish_new])
        .launch()
        .await
        .unwrap();
}
```

## License

This project is licensed under the MIT License.
