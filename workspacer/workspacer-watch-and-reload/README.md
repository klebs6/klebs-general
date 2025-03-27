# workspacer-watch-and-reload

## Overview

`workspacer-watch-and-reload` is a Rust crate enabling efficient file-watching and automatic reloading capabilities within Rust workspaces and crates. Built with an emphasis on asynchronous execution, this crate ensures high responsiveness to file system changes, facilitating automated rebuilds or tests upon detection of significant modifications.

## Features

- **Asynchronous Operation:** Leverages async Rust for non-blocking file monitoring, allowing seamless integration into existing async workflows.
- **Granular Change Detection:** Effectively determines and responds to relevant changes, specifically within `Cargo.toml` and the `src/` directories.
- **Flexible Error Handling:** Employs robust error typing for precise exception management.
- **Dynamic Command Execution:** Interfaces with user-supplied `CommandRunner` implementations to execute custom rebuild or test commands upon file changes.

## Usage

### Trait: `WatchAndReload`
Implement the `WatchAndReload` trait to define how a concrete type should respond to filesystem changes.
```rust
#[async_trait]
impl WatchAndReload for MyCrate {
    type Error = MyError;
    async fn watch_and_reload<'a>(&self, tx: Option<mpsc::Sender<Result<(), Self::Error>>>, runner: Arc<dyn CommandRunner + Send + Sync + 'a>, cancel_token: CancellationToken) -> Result<(), Self::Error> {
        // Implementation
    }
    async fn is_relevant_change(&self, path: &Path) -> bool {
        // Implementation
    }
}
```

### Functions
- **`setup_file_watching`**
  Initialize and configure file watching capabilities.
- **`watch_loop`**
  Continuously monitors for changes, executing the necessary rebuilds or tests.

## Installation
Add the following to your `Cargo.toml`:
```toml
[dependencies]
workspacer-watch-and-reload = "0.1.0"
```

## License

This project is licensed under the MIT License.