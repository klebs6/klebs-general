# `traced-test`

`traced-test` is a Rust crate that provides a procedural macro to automatically handle tracing within your tests, ensuring that test functions use the appropriate tracing instrumentation for logging and span data collection. This crate also enforces the proper use of test attributes by preventing the use of `#[test]` or `#[tokio::test]` alongside the `#[traced_test]` attribute.

## Features

- **Tracing Support in Tests:** Automatically sets up tracing in tests, capturing log and span data for better insight into your test runs.
- **Test Attribute Validation:** Ensures that the `#[traced_test]` attribute is used in place of, not alongside, the `#[test]` or `#[tokio::test]` attributes.
- **Custom Panic Messages:** Provides enhanced context with custom panic messages in case of test failures.
- **Support for `async` and `Result`-returning test functions:** Handles both synchronous and asynchronous test functions, including those returning `Result` types.
- **`should_panic` Support:** Enables the use of the `#[should_panic]` attribute to test expected panics, with improved handling of panic messages.

## Usage

### Basic Usage

To use the `#[traced_test]` attribute in place of `#[test]`, annotate your test functions as follows:

```rust
#[traced_test]
fn my_test() {
    // Your test logic here
}
```

For asynchronous tests, `#[traced_test]` will automatically apply the `#[tokio::test]` attribute:

```rust
#[traced_test]
async fn my_async_test() {
    // Your async test logic here
}
```

### Handling `Result` in Test Functions

Test functions returning `Result` types are supported out-of-the-box. The macro will ensure proper tracing, and if an error occurs, it will be logged and handled as a test failure.

For synchronous tests returning `Result`:

```rust
#[traced_test]
fn my_test() -> Result<(), String> {
    // Your test logic here
    Ok(())
}
```

For asynchronous tests returning `Result`:

```rust
#[traced_test]
async fn my_async_test() -> Result<(), String> {
    // Your async test logic here
    Ok(())
}
```

### Using `#[should_panic]`

The `traced_test` attribute also supports `#[should_panic]`, allowing you to test scenarios where a panic is expected. You can include an optional `expected` message to specify what kind of panic is anticipated.

Synchronous example:

```rust
#[traced_test]
#[should_panic(expected = "This should panic")]
fn my_test() {
    panic!("This should panic");
}
```

Asynchronous example:

```rust
#[traced_test]
#[should_panic(expected = "Async panic")]
async fn my_async_test() {
    panic!("Async panic");
}
```

## Example Scenarios

### Synchronous Tests Returning Nothing

```rust
#[traced_test]
fn sync_test() {
    assert_eq!(2 + 2, 4);
}
```

### Synchronous Tests Returning `Result`

```rust
#[traced_test]
fn sync_test_result() -> Result<(), String> {
    assert_eq!(2 + 2, 4);
    Ok(())
}
```

### Asynchronous Tests Returning Nothing

```rust
#[traced_test]
async fn async_test() {
    let result = async_operation().await;
    assert!(result.is_ok());
}
```

### Asynchronous Tests Returning `Result`

```rust
#[traced_test]
async fn async_test_result() -> Result<(), String> {
    let result = async_operation().await;
    assert!(result.is_ok());
    Ok(())
}
```

### `#[should_panic]` Example

```rust
#[traced_test]
#[should_panic(expected = "This should panic")]
fn should_panic_test() {
    panic!("This should panic");
}
```

## Installation

Add `traced-test` to your `Cargo.toml` dependencies:

```toml
[dependencies]
traced-test = "0.5"
```

## License

This project is licensed under the MIT License.

## Contributing

Contributions are welcome! Please submit issues and pull requests for any improvements or bug fixes.
