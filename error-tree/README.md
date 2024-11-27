# Error Tree

`error-tree` is a Rust procedural macro crate designed to simplify the creation and management of complex error hierarchies in Rust applications. It allows you to define nested error enums in a straightforward and declarative manner, automatically generating `From` implementations and other boilerplate code to facilitate error handling across multiple layers of your application.

## Features

- **Simplified Error Definitions**: Define multiple error enums with nested relationships in a concise syntax.
- **Automatic `From` Implementations**: Automatically generates `From` implementations for error conversions between different error types, even across multiple layers.
- **Customizable `Display` Implementations**: Use the `#[display("...")]` attribute to define custom `Display` messages for your error variants.
- **Custom `PartialEq` Implementations**: Control equality comparisons using the `#[cmp_neq]` attribute for specific variants.
- **Support for Common Traits**: Supports deriving common traits like `Clone` and `PartialEq`.

## Installation

Add `error-tree` to your `Cargo.toml`:

```toml
[dependencies]
error-tree = "0.4.0"
```

## Usage

Import the `error_tree` macro and start defining your error enums using the `error_tree!` macro:

```rust
use error_tree::error_tree;

error_tree! {
    pub enum MyAppError {
        #[display("An unexpected error occurred")]
        UnexpectedError,
        #[display("IO error: {inner}")]
        IoError(std::io::Error),
        #[display("Network error: {url}")]
        NetworkError { url: String },
    }
}
```

This macro will generate:

- The `MyAppError` enum with the specified variants.
- Implementations of the `From` trait for converting from wrapped error types to `MyAppError`.
- A `Display` implementation based on the `#[display("...")]` attributes.

### Example: Defining Nested Error Enums

You can define multiple error enums and specify relationships between them. The macro will automatically generate the necessary `From` implementations for error conversion.

```rust
use error_tree::error_tree;
use std::io;

error_tree! {
    pub enum OuterError {
        InnerError(InnerError),
        #[display("An outer error occurred")]
        OuterVariant,
    }

    pub enum InnerError {
        #[display("An inner IO error: {inner}")]
        IoError(io::Error),
        #[display("A custom inner error")]
        CustomError,
    }
}
```

With this setup, you can convert an `io::Error` directly into an `OuterError`:

```rust
fn cause_io_error() -> Result<(), io::Error> {
    Err(io::Error::new(io::ErrorKind::Other, "Disk not found"))
}

fn handle_error() -> Result<(), OuterError> {
    cause_io_error()?;
    Ok(())
}

fn main() {
    match handle_error() {
        Ok(_) => println!("Success"),
        Err(e) => println!("Error occurred: {}", e),
    }
}
```

### Automatic `From` Implementations

The macro generates `From` implementations that allow seamless conversion between your error types:

```rust
impl From<io::Error> for InnerError {
    fn from(inner: io::Error) -> Self {
        InnerError::IoError(inner)
    }
}

impl From<InnerError> for OuterError {
    fn from(inner: InnerError) -> Self {
        OuterError::InnerError(inner)
    }
}

// This allows:
impl From<io::Error> for OuterError {
    fn from(inner: io::Error) -> Self {
        OuterError::InnerError(InnerError::IoError(inner))
    }
}
```

### Customizing `Display` Messages

Use the `#[display("...")]` attribute to define custom messages for your error variants:

```rust
error_tree! {
    pub enum MyError {
        #[display("Simple error occurred")]
        SimpleError,
        #[display("IO error occurred: {inner}")]
        IoError(std::io::Error),
        #[display("Data error: {data}")]
        DataError { data: String },
    }
}
```

### Controlling `PartialEq` Behavior

You can derive `PartialEq` for your error enums and control comparison behavior for specific variants using the `#[cmp_neq]` attribute:

```rust
error_tree! {
    #[derive(PartialEq)]
    pub enum MyError {
        SimpleError,
        #[cmp_neq]
        NonComparableError(std::io::Error),
        DataError { data: String },
    }
}

let error1 = MyError::NonComparableError(io::Error::new(io::ErrorKind::Other, "Error"));
let error2 = MyError::NonComparableError(io::Error::new(io::ErrorKind::Other, "Error"));

assert_ne!(error1, error2); // Due to #[cmp_neq], these are not equal
```

## Advanced Usage

### Complex Error Hierarchies

`error-tree` excels at handling complex error hierarchies. Here's an example adapted from the crate's tests:

```rust
use error_tree::error_tree;
use std::sync::mpsc;

#[derive(Debug)]
pub struct CpalStreamError;
#[derive(Debug)]
pub struct CpalDeviceNameError;
#[derive(Debug)]
pub struct CpalDevicesError;
#[derive(Debug)]
pub struct CpalHostUnavailable;

error_tree! {
    pub enum PassiveAudioCaptureError {
        FormatError,
        DeviceError(DeviceError),
        IOError(IOError),
        HostError(HostError),
        StreamError(StreamError),
        ChannelError(ChannelError),
    }

    pub enum DeviceError {
        DeviceNotAvailable { device_name: String },
        Basic(CpalDevicesError),
        NameError(CpalDeviceNameError),
    }

    pub enum IOError {
        Basic(std::io::Error),
    }

    pub enum HostError {
        HostUnavailable(CpalHostUnavailable),
    }

    pub enum StreamError {
        StreamError(CpalStreamError),
    }

    pub enum ChannelError {
        ChannelRecvError(mpsc::RecvError),
    }
}

// Usage example
fn cause_device_error() -> Result<(), CpalDeviceNameError> {
    Err(CpalDeviceNameError)
}

fn main() {
    let result: Result<(), PassiveAudioCaptureError> = (|| {
        cause_device_error()?;
        Ok(())
    })();

    match result {
        Ok(_) => println!("Success"),
        Err(e) => println!("Error: {}", e),
    }
}
```

In this example:

- Multiple error enums are defined, representing different components of an audio capture system.
- The macro generates `From` implementations, allowing errors from low-level components (like `CpalDeviceNameError`) to be converted into high-level errors (`PassiveAudioCaptureError`) automatically.
- This simplifies error handling in functions that may return errors from different layers.

## Tests and Examples

The crate includes several tests demonstrating its capabilities:

- **Clone Derivation Test**: Ensures that enums defined with `#[derive(Clone)]` properly implement the `Clone` trait.
- **PartialEq Implementation Test**: Verifies that custom `PartialEq` implementations respect the `#[cmp_neq]` attribute.
- **Display Trait Implementation Test**: Checks that custom `Display` messages are formatted correctly for different variants.

## Limitations

- The macro assumes that the types used in your error variants are valid Rust types that implement necessary traits like `Debug` and `Display` where appropriate.
- For custom types used in wrapped variants, ensure that they implement `Debug` if you want the default `Display` implementation to work correctly.

## Contributing

Contributions are welcome! Please submit issues or pull requests on the [GitHub repository](https://github.com/yourusername/error-tree).

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

---

Feel free to reach out if you have any questions or need assistance using `error-tree`.

## Appendix: Understanding the Macro's Generated Code

To help you understand what the `error_tree!` macro generates, here's an overview based on the crate's code:

### Error Enums and Variants

The macro processes your error enums and their variants, supporting:

- **Basic Variants**: Simple variants without data.
- **Wrapped Variants**: Variants that wrap another error type.
- **Struct Variants**: Variants with named fields.

### `From` Implementations

For each wrapped variant, the macro generates `From` implementations to convert from the wrapped type to the enum containing it. It also generates transitive `From` implementations to allow direct conversion from low-level errors to top-level errors in your hierarchy.

Example generated code:

```rust
impl From<std::io::Error> for IOError {
    fn from(x: std::io::Error) -> Self {
        IOError::Basic(x)
    }
}

impl From<IOError> for PassiveAudioCaptureError {
    fn from(x: IOError) -> Self {
        PassiveAudioCaptureError::IOError(x)
    }
}

// Transitive conversion
impl From<std::io::Error> for PassiveAudioCaptureError {
    fn from(x: std::io::Error) -> Self {
        PassiveAudioCaptureError::IOError(IOError::Basic(x))
    }
}
```

### `Display` Implementations

The macro generates `Display` implementations for your enums, using the `#[display("...")]` attribute if provided. If no `#[display("...")]` attribute is present, it defaults to displaying the variant name.

### Custom `PartialEq` Implementations

If you derive `PartialEq` and use the `#[cmp_neq]` attribute on specific variants, the macro generates a custom `PartialEq` implementation that respects this attribute.

### Attribute Handling

The macro carefully processes attributes to ensure that standard attributes like `#[derive(...)]` are preserved and applied correctly to the generated enums.

## Conclusion

`error-tree` simplifies error handling in Rust applications by reducing boilerplate and providing a clear, declarative way to define complex error hierarchies. By automatically generating necessary implementations, it allows you to focus on your application's logic rather than repetitive error handling code.
