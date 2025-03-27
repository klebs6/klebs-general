# Lightweight Command Runner

## Overview

`lightweight-command-runner` is a Rust crate providing an efficient and asynchronous interface for executing system commands. Built on top of the `tokio` runtime, it facilitates seamless command execution with minimal resource footprint.

## Features
- **Asynchronous Execution**: Leverages Tokio's async capabilities, enabling non-blocking command execution.
- **Cross-Platform Support**: Functions on both Unix and Windows platforms.
- **Streamlined API**: A simple and minimalistic trait-based interface for command execution.

## Usage
To utilize this crate, implement the `CommandRunner` trait for your objects or use the provided `DefaultCommandRunner` struct.

```rust
use lightweight_command_runner::{CommandRunner, DefaultCommandRunner};
use tokio::process::Command;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runner = DefaultCommandRunner;
    let command = Command::new("echo").arg("Hello, World!");

    let handle = runner.run_command(command);
    let output = handle.await??;

    println!("Command executed with output: {:?}", output);

    Ok(())
}
```

## Technical Details
- **Exit Status Handling**: Provides platform-specific implementations to manage command exit statuses.
- This crate is designed for asynchronous execution using the `tokio` runtime.

### Platforms Supported
- Unix-based systems
- Windows

## License
This project is licensed under either the MIT license or Apache License 2.0, at your option.

## Repository
For more details and to contribute, visit the [GitHub repository](https://github.com/klebs6/klebs-general).

## Contact
For any inquiries, reach out to the author, klebs, at tpk3.mx@gmail.com.