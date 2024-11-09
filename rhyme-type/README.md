# Rhyme Type Crate

[![Crates.io](https://img.shields.io/crates/v/rhyme-type.svg)](https://crates.io/crates/rhyme-type)
[![Documentation](https://docs.rs/rhyme-type/badge.svg)](https://docs.rs/rhyme-type)
[![Build Status](https://github.com/yourusername/rhyme-type/workflows/CI/badge.svg)](https://github.com/yourusername/rhyme-type/actions)

The `rhyme-type` crate provides enums and utilities for representing different types of rhymes, rhyme schemes, positions, stresses, and special rhyme forms. It's designed for applications that generate or analyze poetry, lyrics, or other rhyming text.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
  - [Basic Example](#basic-example)
  - [Generating Random Rhyme Types](#generating-random-rhyme-types)
  - [Generating Descriptions](#generating-descriptions)
- [Enums and Structs](#enums-and-structs)
  - [RhymeQuality](#rhymequality)
  - [RhymePosition](#rhymeposition)
  - [RhymeStress](#rhymestress)
  - [RhymeScheme](#rhymescheme)
  - [SpecialRhyme](#specialrhyme)
- [Methods](#methods)
- [Contributing](#contributing)
- [License](#license)
- [Acknowledgments](#acknowledgments)
- [Support](#support)

## Features

- **Rhyme Quality**: Represents the quality of rhymes, such as perfect, slant, eye rhymes, and more.
- **Rhyme Position**: Indicates where the rhymes occur within lines or stanzas.
- **Rhyme Stress**: Captures syllable stress patterns in rhymes.
- **Rhyme Scheme**: Defines specific rhyme schemes, including custom patterns.
- **Special Rhymes**: Includes special rhyme forms that don't fit into other categories.
- **Random Generation**: Generate random rhyme types for testing or creative purposes.
- **Description Generation**: Create detailed descriptions suitable for instructing AI or for documentation.
- **Serializable**: Serialization support using `serde`.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
rhyme-type = "0.1.0"
```

Then, include it in your crate root:

```rust
extern crate rhyme_type;
```

## Usage

### Basic Example

Here's a basic example of how to use the `rhyme-type` crate:

```rust
use rhyme_type::RhymeType;

fn main() {
    // Generate a random rhyme type
    let rhyme_type = RhymeType::random();
    
    // Print the rhyme type details
    println!("{:#?}", rhyme_type);
    
    // Generate a description of the rhyme type
    let description = rhyme_type.generate_description();
    println!("{}", description);
}
```

**Sample Output:**

```
RhymeType {
    quality: Perfect,
    position: Some(End),
    stress: None,
    scheme: Some(Couplet),
    special: None,
}
Use perfect rhymes, where both consonant and vowel sounds match exactly. The rhymes should occur at the end of lines. Follow a couplet rhyme scheme (AABB).
```

### Generating Random Rhyme Types

You can generate a random `RhymeType` using the `random` method:

```rust
use rhyme_type::RhymeType;

let rhyme_type = RhymeType::random();
```

### Generating Descriptions

Generate a description suitable for instructing an AI or for documentation:

```rust
let description = rhyme_type.generate_description();
println!("{}", description);
```

## Enums and Structs

### RhymeQuality

Represents the quality of the rhyme based on sound similarity.

Variants:

- `Perfect`: Exact match of sounds in both consonants and vowels.
- `Slant`: Similar but not identical sounds.
- `Eye`: Words that look like they should rhyme but don't.
- `Identical`: Using the same word twice in rhyming positions.
- `Rich`: Rhyme using homonyms.
- `Wrenched`: Forcing a rhyme by distorting pronunciation.
- `Light`: Rhyming of a stressed syllable with an unstressed syllable.
- `MultiSyllabic`: Rhyming involving multiple syllables.
- `Compound`: Rhyming of two or more compound words.
- `Broken`: Rhyme using a hyphenated word or a word broken across lines.
- `Macaronic`: Rhyme with words from different languages.

### RhymePosition

Indicates the position of the rhyme within the line or stanza.

Variants:

- `End`: Rhyming at the end of lines.
- `Internal`: Rhyming within a single line of verse.
- `Head`: Rhyming of the initial sounds (alliteration).
- `Interlaced`: Rhyming words appear in the middle of one line and at the end of the next.
- `Linked`: Rhyming the end of one stanza with the beginning of the next.
- `Holorhyme`: Rhyming entire lines with each other.
- `Tail`: Rhyming of the final words of lines, especially in concluding lines.

### RhymeStress

Represents syllable stress patterns in the rhyme.

Variants:

- `Masculine`: Rhyming of the final stressed syllable.
- `Feminine`: Rhyming of the final two syllables, with the penultimate syllable stressed.
- `Triple`: Rhyming of the final three syllables, with the first syllable stressed.

### RhymeScheme

Defines specific rhyme schemes.

Variants:

- `Couplet`: AABB
- `Alternate`: ABAB
- `Enclosed`: ABBA
- `Chain`: ABA BCB CDC...
- `Monorhyme`: AAAA
- `Limerick`: AABBA
- `Villanelle`: ABA ABA ABA ABA ABA ABAA
- `SonnetShakespearean`: ABAB CDCD EFEF GG
- `SonnetPetrarchan`: ABBA ABBA CDE CDE
- `TerzaRima`: ABA BCB CDC...
- `Custom(String)`: Custom rhyme scheme.

### SpecialRhyme

Includes special rhyme forms.

Variants:

- `Cross`: Rhyming in a cross pattern (e.g., ABBA).
- `Sporadic`: Irregular rhyme scheme without a set pattern.
- `FreeVerse`: No consistent rhyme.
- `BlankVerse`: Unrhymed iambic pentameter.
- `Enjambment`: Continuing sentences beyond line breaks.
- `Acrostic`: First letters of lines spell out a word.

## Methods

### RhymeType

- `RhymeType::random() -> RhymeType`: Generates a random `RhymeType`.
- `RhymeType::generate_description() -> String`: Generates a description suitable for instructing an AI or for documentation.

### Enums

Each enum provides:

- `random<R: Rng>(rng: &mut R) -> Self`: Generates a random variant.
- `description(&self) -> String`: Provides a description of the variant.

## Contributing

Contributions are welcome! Please follow these steps:

1. Fork the repository on GitHub.
2. Create a new branch for your feature or bugfix.
3. Write your code, including tests.
4. Ensure all tests pass by running `cargo test`.
5. Submit a pull request with a clear description of your changes.

Please make sure to adhere to the existing coding style and include documentation for new features.

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Acknowledgments

- Inspired by the need to model and generate various rhyme types for creative applications.
- Thanks to the Rust community for their support and contributions.

## Support

If you have any questions or issues, please open an issue on the [GitHub repository](https://github.com/yourusername/rhyme-type).
