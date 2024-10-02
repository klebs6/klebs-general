# tracing-setup

**tracing-setup** is a Rust crate that simplifies the configuration and usage of the [`tracing`](https://docs.rs/tracing) crate for instrumenting applications to collect structured, event-based diagnostic information. It provides utilities for buffered logging, dynamic tracing levels, file logging with rotation, and test logging setup functions.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
  - [Buffered Logging](#buffered-logging)
  - [Dynamic Tracing Levels](#dynamic-tracing-levels)
  - [File Logging Configuration](#file-logging-configuration)
  - [Test Logging Utilities](#test-logging-utilities)
- [Modules and Traits](#modules-and-traits)
- [License](#license)

## Features

- **Buffered Logging**: Collect tracing events in a buffer and flush them on demand.
- **Dynamic Tracing Levels**: Change the tracing level at runtime.
- **File Logging**: Log tracing events to files with optional rotation policies.
- **Test Logging Utilities**: Simplify logging setup in tests.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
tracing-setup = "1.0.2"
```

## Usage

### Buffered Logging

Buffered logging allows you to collect tracing events in memory and flush them later. This is useful when you want to defer logging until a certain point in your application, such as after a transaction completes.

#### Setup Buffered Tracing

```rust
use tracing_setup::setup_buffered_tracing;
use tracing::{info, debug};
use tracing::subscriber::set_global_default;

let buffered_subscriber = setup_buffered_tracing(Some("my_tag"));

// Register the subscriber as the global default
set_global_default(buffered_subscriber.clone())
    .expect("Failed to set subscriber");

// Emit some tracing events
info!("This is an info message");
debug!("This is a debug message");

// Flush the buffered events
buffered_subscriber.flush();
```

#### Event Printers

Customize how events are printed when flushed by selecting an `EventPrinter` variant:

- `EventPrinter::FullWithHeader`: Prints the full event with headers.
- `EventPrinter::LogLineAndContents`: Prints the log line and contents.
- `EventPrinter::JustTheContents`: Prints only the event contents (default).

Example:

```rust
use tracing_setup::{setup_buffered_tracing, EventPrinter};
use tracing::subscriber::set_global_default;

let buffered_subscriber = setup_buffered_tracing(None);
buffered_subscriber.buffered_layer.event_printer = EventPrinter::LogLineAndContents;

set_global_default(buffered_subscriber.clone())
    .expect("Failed to set subscriber");

// Emit events and flush
// ...
```

### Dynamic Tracing Levels

Adjust the tracing level at runtime using the `setup_dynamic_tracing` function.

```rust
use tracing_setup::setup_dynamic_tracing;
use tracing::{info, debug, Level};
use tracing_subscriber::EnvFilter;

let reload_handle = setup_dynamic_tracing(Level::INFO);

// Emit some tracing events
info!("This is an info message");
debug!("This debug message should NOT appear at INFO level");

// Change the tracing level to DEBUG at runtime
reload_handle.reload(EnvFilter::new("debug")).unwrap();

// Now debug messages will appear
debug!("This debug message should appear at DEBUG level");
```

### File Logging Configuration

Configure file logging with optional rotation using `FileLoggingConfiguration`.

```rust
use tracing_setup::{FileLoggingConfiguration, init_file_logging};
use tracing::info;
use tracing_appender::rolling::Rotation;
use std::path::PathBuf;

// Configure file logging
let config = FileLoggingConfiguration::new(
    Some(PathBuf::from("app.log")), // Log file path
    tracing::Level::INFO,           // Log level
    Some(Rotation::DAILY),          // Rotation policy
);

// Initialize file logging
init_file_logging(config);

// Now, tracing events will be logged to "app.log" with daily rotation
info!("Application started");
```

### Test Logging Utilities

Simplify logging setup in tests with `init_test_logger`.

```rust
use tracing_setup::init_test_logger;
use tracing::LevelFilter;
use tracing::info;

#[test]
fn my_test() {
    init_test_logger(LevelFilter::Debug, true);

    info!("This is an info message in a test");
}
```

Alternatively, use the `setup_test_logger` macro:

```rust
use tracing_setup::setup_test_logger;
use tracing::info;

#[test]
fn my_test() {
    setup_test_logger!();

    info!("This is an info message in a test");
}
```

## Modules and Traits

### `BufferedLayer`

A tracing subscriber layer that buffers events in memory and can flush them to an output. Useful for deferring logging until a specific point in your application.

#### Example

```rust
use tracing_setup::BufferedLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

let buffered_layer = BufferedLayer::default();
let subscriber = Registry::default().with(buffered_layer.clone());

// Set the subscriber as the default
tracing::subscriber::set_global_default(subscriber)
    .expect("Failed to set subscriber");

// Emit events
tracing::info!("Event 1");
tracing::info!("Event 2");

// Flush events
buffered_layer.flush();
```

### `BufferedSubscriberLayer`

A subscriber that wraps `BufferedLayer` and implements the `Flushable` trait.

### `Flushable` Trait

Defines a `flush` method for types that can flush buffered events.

```rust
pub trait Flushable {
    fn flush(&self);
}
```

### `HandleBufferWriteEvent` Trait

Handles writing events to the buffer in different formats.

Implementations:

- `FullWithHeader`: Prints full event details with headers.
- `LogLineAndContents`: Prints the log line and contents.
- `JustTheContents`: Prints only the event contents.

### `EventPrinter` Enum

Specifies the format for printing events when flushing.

```rust
#[derive(Debug, Clone)]
pub enum EventPrinter {
    FullWithHeader,
    LogLineAndContents,
    JustTheContents,
}
```

### `FileLoggingConfiguration`

Configures file logging with options for log path, level, rotation, and whether the log is temporary (deleted when the configuration is dropped).

#### Fields

- `log_path: Option<PathBuf>`: Path to the log file.
- `log_level: Level`: Logging level.
- `rotation: Option<Rotation>`: Log rotation policy.
- `temporary: bool`: Indicates if the log file should be deleted when dropped.

#### Methods

- `new`: Creates a new configuration.
- `new_temporary`: Creates a new temporary configuration.
- `default`: Provides default configuration.
- `default_temporary`: Provides default temporary configuration.
- `create_writer`: Creates a writer based on the configuration.

### `init_file_logging`

Initializes file logging based on a `FileLoggingConfiguration`.

```rust
pub fn init_file_logging(config: FileLoggingConfiguration)
```

### Utility Functions

- `configure_tracing`: Initializes the logging subscriber with environment filter.
- `init_default_file_logging`: Initializes file logging with default configuration.
- `init_test_logger`: Initializes a test logger with specified level.
- `init_test_logger_with_max_level_filter`: Initializes a test logger with maximum level filter.

## License

This project is licensed under the [MIT License](https://opensource.org/licenses/MIT).

---

**Note**: Ensure that all dependencies such as `tracing`, `tracing_subscriber`, `tracing_appender`, and `colored` are included in your `Cargo.toml` to use the features demonstrated in this README.

Feel free to contribute to this crate by opening issues or submitting pull requests.
