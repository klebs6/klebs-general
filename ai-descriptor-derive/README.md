# ai-descriptor-derive

**ai-descriptor-derive** is a Rust procedural macro crate that automatically implements the `AIDescriptor` trait for enums based on the `#[ai("...")]` attributes provided on each variant. This helps eliminate boilerplate code and makes your codebase cleaner and more maintainable.

## Features

- **Automatic Trait Implementation**: Derive `AIDescriptor` for your enums, and the macro will generate the implementation based on the `#[ai("...")]` attributes.
- **Compile-Time Error Checking**: If a variant is missing the `#[ai("...")]` attribute, the macro will produce a compile-time error.
- **Seamless Integration**: Works well with other procedural macros like `RandomConstructible`.

## Getting Started

### Add Dependency

Add the following to your `Cargo.toml`:

```toml
[dependencies]
ai-descriptor-derive = "0.1.0"
```

### Import the Crate

In your Rust file, import the `AIDescriptor` macro:

```rust
use ai_descriptor_derive::AIDescriptor;
```

## Usage

### Deriving `AIDescriptor` for Enums

You can automatically implement `AIDescriptor` for your enums by using the `#[derive(AIDescriptor)]` macro and adding `#[ai("...")]` attributes to each variant.

#### Example

```rust
use ai_descriptor_derive::AIDescriptor;
use std::borrow::Cow;

#[derive(AIDescriptor)]
enum Emotion {
    #[ai("A feeling of great pleasure or happiness.")]
    Joy,
    #[ai("A strong feeling of displeasure or hostility.")]
    Anger,
    #[ai("A feeling of sadness or grief.")]
    Sadness,
}

impl Emotion {
    fn ai(&self) -> Cow<'_, str> {
        // Generated automatically by the macro
    }
}
```

### Using the `ai()` Method

After deriving `AIDescriptor`, you can use the `ai()` method:

```rust
fn main() {
    let emotion = Emotion::Joy;
    println!("Description: {}", emotion.ai());
}
```

#### Output

```
Description: A feeling of great pleasure or happiness.
```

## Attributes

### `#[ai("...")]`

- **Applies to**: Enum variants.
- **Purpose**: Provides a description or associated string with the variant.
- **Required**: Yes, for all variants when using `#[derive(AIDescriptor)]`.

## Limitations

- **Enums Only**: The `AIDescriptor` derive macro only works with enums.
- **Mandatory `#[ai]` Attributes**: All variants must have the `#[ai("...")]` attribute; otherwise, a compile-time error will occur.

## Integration with Other Crates

- **RandomConstructible**: The `ai-descriptor` crate can be used alongside the `random-constructible` crate to provide both random instantiation and AI descriptions for your enums.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contribution

Contributions are welcome! Please open an issue or submit a pull request on GitHub.

## Acknowledgments

- Inspired by the need to simplify the implementation of descriptive methods for enums in Rust.
- Utilizes the `proc-macro` crate for procedural macros and `syn` and `quote` for parsing and generating Rust code.

# Contact

For questions or suggestions, feel free to open an issue or contact the maintainer.

---

Happy coding!
