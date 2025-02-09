# Command Runner

Command Runner is a lightweight asynchronous command execution library that provides a simple trait-based interface for running system commands using [Tokio](https://tokio.rs). With a default implementation and the ability to customize, it lets you integrate command execution easily into your asynchronous Rust applications.

## Features

- **Asynchronous Execution:** Run system commands using async/await.
- **Trait-Based API:** Easily swap out or extend the command runner with your own implementation.
- **Cross-Platform Support:** Includes Unix/Windows specific extensions for exit status handling.
- **Ergonomic Design:** Clean, minimal API for straightforward command execution.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
command-runner = "0.1.0"
```

## Usage

Here's a basic example using the default implementation:

```rust
use command_runner::{CommandRunner, DefaultCommandRunner};
use tokio::process::Command;

#[tokio::main]
async fn main() {
    let runner = DefaultCommandRunner;

    // Configure the command based on the target OS
    let mut cmd = if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.args(["/C", "echo hello"]);
        c
    } else {
        let mut c = Command::new("echo");
        c.arg("hello");
        c
    };

    match runner.run_command(cmd).await {
        Ok(output) => {
            println!("Command executed successfully!");
            println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        },
        Err(e) => {
            eprintln!("Command execution failed: {}", e);
        }
    }
}
```

## License

This project is dual-licensed under either the [MIT license](LICENSE-MIT) or the [Apache License, Version 2.0](LICENSE-APACHE), at your option.

## Contributing

Contributions are welcome! Please check out the [repository](https://github.com/klebs6/klebs-general) for details.
