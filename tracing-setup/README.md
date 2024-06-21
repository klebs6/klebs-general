# `tracing-setup`

`tracing-setup` is a Rust crate that provides a structured way to configure and manage tracing with buffered logging. It introduces a `BufferedLayer` that allows capturing and storing tracing events in a buffer, which can be flushed and printed as needed.

## Features

- **Buffered Logging**: Captures tracing events in a buffer.
- **Customizable Output**: Supports different print types for events.
- **Flushing Mechanism**: Flushes buffered events to the console.
- **Subscriber Integration**: Easily integrates with `tracing` subscribers.

## Usage

### Add Dependency

Add the following to your `Cargo.toml`:

```toml
[dependencies]
tracing = "0.1.40"
tracing-subscriber = { version = "0.3", default-features = false, features = ["fmt", "std", "env-filter"] }
colored = "2.1.0"
```

## Example

Here's an example of how to use tracing-setup:

```rust
use tracing_setup::{configure_tracing, setup_buffered_tracing};
use tracing::{info, Level};
use std::sync::Arc;

fn main() {
    configure_tracing();

    let buffered_subscriber = setup_buffered_tracing(Some("example-tag"));
    
    // Use the tracing macros to generate events
    info!("This is an informational message");

    // Flush the buffered events
    buffered_subscriber.flush();
}
```

If we also use `traced-test`, we can do the following:

```rust
use traced_test::traced_test;

#[traced_test]
fn example_test() -> Result<(),()> {
    info!("running an example test!");
    assert_eq!(1 + 1, 2);

    Ok(())
}
```

## Structs and Traits

### `BufferedLayer`

A layer that captures tracing events and stores them in a buffer.

```rust
pub struct BufferedLayer {
    tag: Option<String>,
    buffer: Arc<Mutex<Vec<String>>>,
}
```

#### Methods

`new(tag: &str) -> Self`: Creates a new BufferedLayer with a specified tag.

`flush(&self)`: Flushes the buffered events to the console.

### `BufferedSubscriberLayer`

A subscriber layer that includes a BufferedLayer.

```rust
pub struct BufferedSubscriberLayer<S> {
    inner: tracing_subscriber::layer::Layered<BufferedLayer, S>,
    buffered_layer: Arc<BufferedLayer>,
}
```
#### Methods

`flush(&self)`: Flushes the buffered events using the inner BufferedLayer.


### `Flushable`

A trait for flushing buffered events.

```rust
pub trait Flushable {
    fn flush(&self);
}
```

## Functions
`configure_tracing`

Initializes the logging subscriber.

```rust
pub fn configure_tracing() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        let filter = tracing_subscriber::EnvFilter::from_default_env()
            .add_directive(Level::DEBUG.into());  
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .init();
    });
}
```

`setup_buffered_tracing`

Sets up a buffered tracing subscriber with an optional tag.

```rust
pub fn setup_buffered_tracing(tag: Option<&str>) -> Arc<BufferedSubscriberLayer<Registry>> {
    let buffered_layer = match tag { 
        Some(tag) => BufferedLayer::new(tag), 
        None => BufferedLayer::default() 
    };
    Arc::new(BufferedSubscriberLayer {
        inner: Registry::default().with(buffered_layer.clone()),
        buffered_layer: buffered_layer.into(),
    })
}
```

# License

This crate is licensed under the MIT License.
