# traced-test

A procedural macro for Rust that enhances your tests with tracing capabilities, capturing logs and spans for better debugging and test output. `traced-test` allows you to automatically capture and display logs when tests fail, making it easier to diagnose issues.

**Note:** This crate is currently in flux. Some features are still being worked out. We have recently found and fixed several unexpected behaviors. It is possible there are more TODO.  Please see the [Limitations](#limitations) section for more details.

## Features

- **Automatic Log Capture**: Captures logs produced during tests and displays them when tests fail.
- **Integration with `tracing` Crate**: Works with the [`tracing`](https://crates.io/crates/tracing) crate for structured logging and diagnostics.
- **Customizable Retention Policies**: Configure when logs should be retained (e.g., on test failure).
- **Support for Asynchronous and Synchronous Tests**: Works with both async and sync tests.
- **Handling of Expected Failures**: Supports tests that are expected to fail, with optional failure messages.

## Installation

Add `traced-test` to your `Cargo.toml` under `[dev-dependencies]`:

```toml
[dev-dependencies]
traced-test = "1.0.2"
```

## Usage

Annotate your tests with the `#[traced_test]` attribute instead of the standard `#[test]` attribute:

```rust
use traced_test::traced_test;
use tracing::{info, warn, error};

#[traced_test]
fn my_test() {
    info!("This is an informational message.");
    warn!("This is a warning message.");
    error!("This is an error message.");
    assert!(false, "This test will fail.");
}
```

When the test fails, `traced-test` will display the captured logs, along with any panic messages and backtraces, to help you diagnose the issue.

### Example Output

```
===== BEGIN_TEST: my_test =====
thread 'my_test' panicked at 'assertion failed: false: This test will fail.', src/main.rs:10:5
-----------------------------------------------trace_events: my_test]
This is an informational message.
This is a warning message.
This is an error message.
===== Backtrace for test: my_test =====
[... backtrace output ...]
===== END_TEST: my_test =====
```

## Handling Expected Failures

You can mark a test as expected to fail using the `#[should_fail]` attribute:

```rust
use traced_test::traced_test;
use traced_test::should_fail;
use tracing::error;

#[traced_test]
#[should_fail]
fn test_expected_failure() {
    error!("An error occurred.");
    panic!("This test is expected to fail.");
}
```

You can also specify an expected failure message:

```rust
use traced_test::traced_test;
use traced_test::should_fail;
use tracing::error;

#[traced_test]
#[should_fail(message = "Expected failure message")]
fn test_expected_failure_with_message() {
    error!("An error occurred.");
    panic!("Expected failure message");
}
```

## Important Notes

### Disclaimer

**Please note that `traced-test` is currently under development, and some features are still being worked out:**

- **`--nocapture` Flag:** Using the `--nocapture` flag with `cargo test` is not currently supported and may lead to unexpected behavior. It's recommended to run tests without this flag when using `traced-test`.

- **Other Limitations:** There may be other minor issues as the crate is still being refined. Simple usage should work as expected, but please report any problems you encounter.

### Using `#[traced_test]` Correctly

- Replace `#[test]` with `#[traced_test]` on your test functions. Do not use both attributes on the same function.
- For async tests, `#[traced_test]` will automatically handle the async context. You don't need to add `#[tokio::test]` or similar attributes.

## Integration with `tracing`

Ensure that you have the `tracing` crate added to your dependencies:

```toml
[dependencies]
tracing = "0.1"
```

You can then use `tracing` macros like `info!`, `warn!`, `error!` in your tests.

## Limitations

- **`--nocapture` Flag:** As mentioned, avoid using `--nocapture` with `cargo test` when using `traced-test`, as it can interfere with output capturing and lead to interleaved and confusing logs.
  
- **Compatibility with Standard `#[test]` Attribute:** Mixing `#[traced_test]` with standard `#[test]` functions in the same test suite has not been thoroughly tested and may cause issues.

- **Panic Handling:** While `traced-test` attempts to capture panics and ensure logs are flushed before panic messages are displayed, there might be edge cases where this doesn't work perfectly.

- **Global State:** Be cautious when using global state or static variables in your tests, as they can interfere with parallel test execution.

## Contributing

Contributions are welcome! Please submit issues and pull requests on the [GitHub repository](https://github.com/yourusername/traced-test).

If you encounter any problems or have suggestions for improvements, please open an issue to let us know.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
