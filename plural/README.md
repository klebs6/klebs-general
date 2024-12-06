# `plural`

The `plural` crate provides a unified interface for generating and working with plural forms of enum variants in Rust. It combines the functionality of the `plural-derive` and `plural-trait` crates, offering a seamless experience for deriving and using pluralization in your projects.

## Features

- **Derive Procedural Macro**: Use the `#[derive(Plural)]` attribute to automatically generate pluralization logic for enums.
- **Custom Pluralization**: Customize plural forms with the `#[plural("...")]` attribute.
- **Default Pluralization**: Automatically converts enum variant names to lowercase words and appends "s" when no custom plural is provided.
- **Supports Empty Enums**: Safely handles enums with no variants.
- **Unified API**: All functionality is exposed through the `plural` crate for ease of use.

## Getting Started

### Add the Crate

Add `plural` to your `Cargo.toml`:

```toml
[dependencies]
plural = "0.1.0"
```

### Example Usage

#### Define an Enum with Pluralization

```rust
use plural::{Plural, PluralDisplay};

#[derive(Plural, Debug)]
pub enum PoeticForms {
    #[plural("alcaic stanzas")]
    AlcaicStanza,
    #[plural("burns stanzas")]
    BurnsStanza,
    BlankVerse, // Default: "blank verses"
    Couplet,    // Default: "couplets"
    Haiku,      // Default: "haikus"
}

fn main() {
    let form = PoeticForms::Haiku;
    println!("Plural: {}", form.plural()); // Output: "Plural: haikus"
}
```

#### Custom Pluralization

```rust
#[derive(Plural, Debug)]
pub enum CustomEnum {
    #[plural("special cases")]
    SpecialCase,
    #[plural("irregular")]
    IrregularForm,
}
```

#### Empty Enum Support

```rust
#[derive(Plural, Debug)]
pub enum EmptyEnum {}

// Empty enums are supported safely and are uninhabited at runtime.
```

#### Using the `PluralDisplay` Trait

The `PluralDisplay` trait provides a `plural()` method:

```rust
use PoeticForms::*;

assert_eq!(AlcaicStanza.plural(), "alcaic stanzas");
assert_eq!(BlankVerse.plural(), "blank verses");
assert_eq!(Couplet.plural(), "couplets");
```

### Test Output

Run the included test suite to verify functionality:

```sh
cargo test
```

## How It Works

1. **Custom Attribute Parsing**: The procedural macro parses `#[plural("...")]` attributes to specify custom plural forms.
2. **Default Pluralization**: For variants without custom attributes, the macro automatically generates a plural form by converting CamelCase to lowercase words and appending "s".
3. **Unified API**: The crate combines `plural-derive` for deriving implementations and `plural-trait` for using the `PluralDisplay` trait.

## Benefits of the `plural` Crate

- Simplifies dependency management by combining `plural-derive` and `plural-trait`.
- Provides a single entry point for both deriving and using pluralization functionality.
- Ideal for projects where enums require consistent pluralization logic.

## Limitations

- The crate currently supports only enums. Attempting to derive `Plural` for other types will result in a compile-time error.
- Default pluralization is basic and may not handle all linguistic edge cases.

## Contributing

Contributions are welcome! If you encounter bugs, have suggestions, or want to extend the functionality, feel free to open an issue or submit a pull request.

1. Fork the repository.
2. Create your feature branch: `git checkout -b feature-name`.
3. Commit your changes: `git commit -m 'Add feature'`.
4. Push to the branch: `git push origin feature-name`.
5. Open a pull request.

## License

This project is licensed under the MIT License. See the [LICENSE](./LICENSE) file for details.

## Acknowledgments

Special thanks to the Rust community for their resources and guidance in building procedural macros.

---

For more detailed documentation, refer to the individual `plural-derive` and `plural-trait` crates.
