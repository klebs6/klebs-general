# `plural-derive`

`plural-derive` is a procedural macro crate for Rust that simplifies generating plural forms for enum variants. With this crate, you can derive the `PluralDisplay` trait for enums, enabling a method to retrieve the pluralized version of each variant.

## Features

- **Customizable Pluralization**: Use the `#[plural("...")]` attribute to specify custom plural forms for enum variants.
- **Default Pluralization**: Automatically generates default plural forms by converting CamelCase variants to lowercase words and appending "s".
- **Handles Special Cases**: Supports special characters, spaces, and irregular pluralization rules.
- **Type Safety**: Provides compile-time enforcement for pluralization logic, ensuring correctness.
- **Support for Empty Enums**: Generates safe, uninhabited code for empty enums.

## Example Usage

### Add Dependencies

Add `plural-derive` and `plural-trait` to your `Cargo.toml`:

```toml
[dependencies]
plural-derive = "0.1.0"
plural-trait = "0.1.0"
```

### Define an Enum

```rust
use plural_derive::Plural;
use plural_trait::PluralDisplay;

#[derive(Plural, Debug)]
pub enum PoeticForms {
    #[plural("alcaic stanzas")]
    AlcaicStanza,
    #[plural("ballads")]
    Ballad,
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

### Custom Pluralization

You can specify irregular or custom plural forms using the `#[plural("...")]` attribute:

```rust
#[derive(Plural, Debug)]
pub enum CustomEnum {
    #[plural("special cases")]
    SpecialCase,
    #[plural("irregular")]
    IrregularForm,
}
```

### Empty Enum Handling

Enums with no variants are supported safely:

```rust
#[derive(Plural, Debug)]
pub enum EmptyEnum {}
```

Attempting to use an instance of `EmptyEnum` will result in a compile-time error.

### Test Output

The `PluralDisplay` trait provides the `plural()` method to retrieve the pluralized form of an enum variant:

```rust
use PoeticForms::*;

assert_eq!(AlcaicStanza.plural(), "alcaic stanzas");
assert_eq!(BlankVerse.plural(), "blank verses");
assert_eq!(Couplet.plural(), "couplets");
```

## Testing

Run the included test suite to verify functionality:

```sh
cargo test
```

## How It Works

1. **Custom Attribute Parsing**: The `#[plural("...")]` attribute allows specifying custom plural forms.
2. **Default Pluralization Logic**: For variants without a `#[plural(...)]` attribute, the macro converts CamelCase to lowercase words and appends "s".
3. **Trait Implementation**: The procedural macro generates an implementation of the `PluralDisplay` trait, which provides the `plural()` method.

## Limitations

- Only supports enums. Attempting to derive `Plural` for structs or other data types will result in a compile-time error.
- Default pluralization is basic and may not handle all linguistic nuances.

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
