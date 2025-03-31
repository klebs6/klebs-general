# sync-run-async
This crate provides a way to synchronously run async code.

## Overview
`sync-run-async` is a highly efficient utility for executing asynchronous futures from synchronous contexts in Rust applications. It leverages existing Tokio runtime handles when available and creates new ones when necessary, eliminating runtime nesting concerns. This crate is particularly beneficial for developers who need a reliable way to bridge synchronous and asynchronous code without introducing runtime complexity.

## Features
- **Seamless Invocation**: Automatically detects existing Tokio runtimes to execute asynchronous code, reducing overhead and complexity.
- **Robust Error Handling**: Isolates potential panics, ensuring your application remains stable even when threading issues occur.
- **Production-Ready**: Extensively tested within production environments, providing assurance of its reliability and performance.

## Usage
### Basic Usage
To use `sync-run-async`, simply call the `sync_run_async` function with your future:

```rust
use workspacer_3p::sync_run_async;

async fn my_async_function() -> i32 {
    42
}

fn main() {
    let result = sync_run_async(my_async_function());
    println!("Result: {}", result);
}
```

### Application
Ideal for scenarios where asynchronous operations must be launched from synchronous code but need to safely handle runtime context determination. Perfect for scenarios like testing or embedding asynchronous operation in traditionally synchronous codebases.

## Installation
Add the following to your `Cargo.toml`:
```toml
[dependencies]
sync-run-async = "0.5.0"
```

## Contributions
This crate is open-source and contributors are welcomed. The code repository is hosted on [GitHub](https://github.com/klebs6/klebs-general). Please adhere to the contributor guidelines.

## License
Licensed under the MIT License. See the LICENSE file for details.
