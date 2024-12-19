
# language-enum

`language-enum` is a robust, exhaustive enum representing a wide variety of global and regional languages. It’s designed for use in applications that require internationalization (i18n), localization, and handling of multiple languages.

## Features

- **Exhaustive Language Support**: Covers major world languages, regional languages, and some languages spoken by smaller populations.
- **Serialization & Deserialization**: Comes with built-in support for `serde` to easily serialize and deserialize languages.
- **Flexible `Other` Variant**: Allows specifying languages not covered in the predefined enum through the `Other(String)` variant.
- **No Dependencies**: Apart from optional `serde` support, the crate is dependency-free.

## Installation

Add `language-enum` to your `Cargo.toml`:

```toml
[dependencies]
language-enum = "0.2"
```

To enable serde support for serialization and deserialization:

```toml
[dependencies]
language-enum = { version = "0.1", features = ["serde"] }
```

## Usage

```rust
use language_enum::Language;

fn main() {
    let lang = Language::English;
    println!("Selected language: {:?}", lang);

    // Using the `Other` variant for an unsupported language
    let other_lang = Language::Other("Klingon".to_string());
    println!("Other language: {:?}", other_lang);

    let random_lang         = Language::random(); // this is weighted by number of speakers on earth
    let uniform_random_lang = Language::uniform();
}

```

## Enum Variants

The Language enum includes a wide range of languages such as:

Major Languages: English, Spanish, Chinese (Mandarin), Arabic, Russian, etc.
Regional Languages: Breton, Hawaiian, Basque, and others.
Indigenous Languages: Cherokee, Navajo, Maori, and more.
It also includes an Other(String) variant to cover any languages not explicitly listed.

## Features

Serialization: Serialize and deserialize the enum with serde.
Exhaustive: Covers a broad range of languages for various applications.
Custom Language Support: Use Language::Other(String) for languages not in the predefined set.
License

## Licensed under either of:

Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)
at your option.

## Contribution

Unless explicitly stated otherwise, any contribution intentionally submitted for inclusion in the work by you shall be dual licensed as above, without any additional terms or conditions.
