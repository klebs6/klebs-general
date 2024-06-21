
# `traced-test`

`traced-test` is a Rust crate providing a procedural macro to enhance test functions with tracing capabilities. It ensures that test functions are traced and that their events are buffered and flushed appropriately.

## Features

- **Automatic Tracing**: Automatically sets up tracing for test functions.
- **Error Handling**: Ensures proper error handling and flushing of trace logs upon test failures.
- **Simple Integration**: Easy to integrate with existing test functions by replacing the `#[test]` attribute.

## Usage

### Add Dependency

Add the following to your `Cargo.toml`:

```toml
[dependencies]
traced-test = "0.1"
tracing-setup = "0.1"
tracing = "0.1"
syn = { version = "1.0", features = ["full"] }
quote = "1.0"
```

## Example

Here's an example of how to use traced-test:

```rust
use traced_test::traced_test;

#[traced_test]
fn example_test() -> Result<(),()> {
    info!("running an example test!");
    assert_eq!(1 + 1, 2);

    Ok(())
}
```

Note that as of version `0.2.0`, the #[traced_test] macro requires for the
test function to return a Result.

## License

This crate is licensed under the MIT license.
