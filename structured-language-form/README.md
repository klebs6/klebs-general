# Structured Language Form Crate

The `structured_language_form` crate provides a comprehensive and structured way to work with various poetic and literary forms, enabling applications in natural language processing, creative writing, and education. Whether you're generating poetry, analyzing text, or exploring structured language, this crate offers a robust foundation for handling a variety of structured language forms.

## Features

- **Comprehensive Enum**: Represents a wide array of poetic and literary forms, including sonnets, odes, haikus, and many more.
- **AI-Optimized Descriptions**: Each variant is annotated with an `#[ai]` attribute, providing concise descriptions of the form.
- **Random Generation**: The crate includes utilities for random generation of structured language forms, with full support for seeded RNG to ensure reproducibility.
- **Serialization**: Supports serialization and deserialization for integration with external systems.

## Usage

### Add to `Cargo.toml`

```toml
[dependencies]
structured_language_form = "0.1.0"
rand = "0.8"  # Required for random generation
serde = { version = "1.0", features = ["derive"] }  # Optional for serialization
```

### Example Code

```rust
use structured_language_form::StructuredLanguageForm;
use rand::SeedableRng;
use rand::rngs::StdRng;

fn main() {
    // Generate a specific language form
    let haiku = StructuredLanguageForm::Haiku;
    println!("Selected form: {:?}", haiku);

    // Generate a random language form with a seeded RNG
    let mut rng = StdRng::seed_from_u64(42);
    let random_form = StructuredLanguageForm::random_with_rng(&mut rng);
    println!("Random form: {:?}", random_form);

    // Serialize and deserialize (optional, requires serde feature)
    #[cfg(feature = "serde")]
    {
        let serialized = serde_json::to_string(&random_form).unwrap();
        println!("Serialized: {}", serialized);

        let deserialized: StructuredLanguageForm =
            serde_json::from_str(&serialized).unwrap();
        println!("Deserialized: {:?}", deserialized);
    }
}
```

### Enum Variants

Below is a sample of the structured language forms included in the crate:

- **AlcaicStanza** -- "4 lines, classical meter"
- **Ballad** -- "Quatrains, ABAB or ABCB"
- **BlankVerse** -- "Unrhymed iambic pentameter"
- **Haiku** -- "3 lines, 5-7-5 syllables"
- **Sonnet** -- "14-line poems with a specific rhyme scheme, often about love"
- **Villanelle** -- "19 lines, ABA ABA ABA ABA ABA ABAA"
- ... and many more.

Refer to the source code for the full list.

## Random Generation

The crate supports deterministic random generation using the `rand` crate. This feature is ideal for testing and applications where reproducibility is crucial.

Example:

```rust
let mut rng = StdRng::seed_from_u64(12345);
let random_form = StructuredLanguageForm::random_with_rng(&mut rng);
```

## Serialization and Deserialization

With the `serde` feature enabled, you can serialize and deserialize `StructuredLanguageForm` instances into formats like JSON:

```rust
use serde_json;
use structured_language_form::StructuredLanguageForm;

let form = StructuredLanguageForm::Sonnet;
let serialized = serde_json::to_string(&form).unwrap();
let deserialized: StructuredLanguageForm = serde_json::from_str(&serialized).unwrap();
```

## Integration

This crate integrates well with AI tools, natural language processing frameworks, and creative writing applications. The annotations (`#[ai]`) make it especially suitable for AI-driven systems that require metadata or descriptions of the variants.

## Contributing

Contributions to the crate are welcome! Please open an issue or submit a pull request on the [GitHub repository](https://github.com/your-username/structured_language_form).

## License

This crate is licensed under the MIT License. See the `LICENSE` file for details.
