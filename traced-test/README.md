
# traced-test

`traced-test` is a Rust procedural macro crate that provides the `#[traced_test]` attribute for enhancing your test functions with tracing capabilities. It integrates seamlessly with the `tracing` crate to capture and display logs specific to individual tests, making debugging and analyzing test outputs more efficient.

## Features

- **Automatic Tracing Setup**: Automatically initializes a tracing subscriber for each test, capturing logs emitted during test execution.
- **Supports Async Tests**: Works with both synchronous and asynchronous test functions.
- **Custom Failure Messages**: Provides a `#[should_fail]` attribute to specify expected failure messages.

## Installation

Add `traced-test` to your `Cargo.toml`:

```toml
[dev-dependencies]
traced-test = "1.0.0"
```

## Usage

### Basic Usage

Simply annotate your test functions with `#[traced_test]` instead of `#[test]` or `#[tokio::test]`.

```rust
use traced_test::traced_test;
use tracing::info;

#[traced_test]
fn my_test() {
    info!("This is a traced test");
    assert!(true);
}
```

For asynchronous tests:

```rust
use traced_test::traced_test;
use tracing::info;

#[traced_test]
async fn my_async_test() {
    info!("This is an async traced test");
    assert!(true);
}
```

### Expecting Failures

If you expect a test to fail, use the `#[should_fail]` attribute. You can optionally provide a custom failure message.

```rust
use traced_test::traced_test;
use traced_test::should_fail;

#[traced_test]
#[should_fail]
fn test_should_fail() {
    panic!("This test should fail");
}

#[traced_test]
#[should_fail(message = "Expected failure")]
fn test_should_fail_with_message() {
    panic!("Expected failure");
}
```

## How It Works

- **Tracing Initialization**: The macro sets up a local tracing subscriber for each test, capturing all logs emitted during the test's execution.
- **Log Flushing**: On test completion, logs are flushed based on the test's success or failure and the configured tracing options.
- **Error Handling**: For tests that return `Result`, the macro properly handles `Ok` and `Err` variants, matching expected failure messages if provided.

## Examples

### Capturing Logs on Failure

By default, logs are output only when a test fails. This helps reduce noise in test outputs.

```rust
use traced_test::traced_test;
use tracing::info;

#[traced_test]
fn test_failure_logs() {
    info!("This will be logged if the test fails");
    assert!(false, "Intentional failure");
}
```

### Async Tests with Expected Failures

```rust
use traced_test::traced_test;
use traced_test::should_fail;
use tracing::info;

#[traced_test]
#[should_fail(message = "Async failure")]
async fn async_test_should_fail() {
    info!("This async test is expected to fail");
    panic!("Async failure");
}
```

## Error Handling

When using `#[should_fail]`, you can provide an expected failure message. The test will pass only if it fails with the specified message.

```rust
use traced_test::traced_test;
use traced_test::should_fail;

#[traced_test]
#[should_fail(message = "Specific failure message")]
fn test_with_expected_failure() {
    panic!("Specific failure message");
}
```

If the test does not fail or fails with a different message, it will be marked as failed.

### Configuration Options

You can control the tracing behavior using the `trace` option:

- `trace = true`: Always output logs, regardless of test outcome.
- `trace = false`: Never output logs.
- By default, logs are output only on test failure.

NOTE: this feature is not fully implemented in 1.0.0 but remains todo. 

Pull requests finishing the feature are welcome. 

For now, to force tracing ON we can assert at the end of the test. 

To force tracing OFF we can fallback to #[test] instead of #[traced_test].

```rust
use traced_test::traced_test;
use tracing::info;

#[traced_test(trace = true)]
fn always_trace_test() {
    info!("Logs will always be output");
    assert!(true);
}
```

## Contributing

Contributions are welcome! Please open an issue or submit a pull request on [GitHub](https://github.com/yourusername/traced-test).

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
