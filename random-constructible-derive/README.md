# Random Constructible Derive

[![Crates.io](https://img.shields.io/crates/v/random-constructible-derive.svg)](https://crates.io/crates/random-constructible-derive)
[![Documentation](https://docs.rs/random-constructible-derive/badge.svg)](https://docs.rs/random-constructible-derive)
[![License](https://img.shields.io/crates/l/random-constructible-derive.svg)](https://crates.io/crates/random-constructible-derive)

The `random-constructible-derive` crate provides a procedural macro to automatically implement the `RandomConstructible` trait from the [`random-constructible`](https://crates.io/crates/random-constructible) crate for your enums. It simplifies the process of making your enums randomizable with weighted probabilities.

## Features

- Automatically generates implementations for the `RandomConstructible` trait.
- Supports specifying default unnormalized construction probabilities via attributes.
- Ensures that the generated code is efficient and leverages lazy initialization.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
random-constructible = "0.1.0"
random-constructible-derive = "0.1.0"
```

Ensure you also include the `rand` crate if you're not already using it:

```toml
[dependencies]
rand = "0.8"
```

## Usage

Simply derive `RandomConstructible` on your enum and optionally specify default probabilities using the `#[default_unnormalized_construction_probability = value]` attribute.

### Example

```rust
use random_constructible::{RandomConstructible, RandomConstructibleProbabilityMapProvider};
use random_constructible_derive::RandomConstructible;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(RandomConstructible, Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum PotionEffectType {
    #[default_unnormalized_construction_probability = 4.0]
    Healing,
    Enhancement,
    #[default_unnormalized_construction_probability = 2.0]
    Poison,
    Invisibility,
}

impl Default for PotionEffectType {
    fn default() -> Self {
        Self::Healing
    }
}

fn main() {
    let random_effect = PotionEffectType::random();
    println!("Random Potion Effect: {:?}", random_effect);
}
```

### Specifying Default Probabilities

By default, each variant has an unnormalized probability of `1.0`. You can adjust this by adding the `#[default_unnormalized_construction_probability = value]` attribute to any variant.

```rust
#[derive(RandomConstructible, Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum Weather {
    #[default_unnormalized_construction_probability = 5.0]
    Sunny,
    #[default_unnormalized_construction_probability = 2.0]
    Rainy,
    Cloudy,
    Stormy,
}
```

### Custom Probability Providers

You can create custom probability providers by implementing the `RandomConstructibleProbabilityMapProvider` trait.

```rust
struct DesertEnvironment;

impl RandomConstructibleProbabilityMapProvider<Weather> for DesertEnvironment {
    fn probability_map(&self) -> Arc<HashMap<Weather, f64>> {
        let mut map = HashMap::new();
        map.insert(Weather::Sunny, 8.0);
        map.insert(Weather::Rainy, 1.0);
        Arc::new(map)
    }
}

fn main() {
    let weather = Weather::random_with_probabilities(&DesertEnvironment);
    println!("Weather in Desert: {:?}", weather);
}
```

## Limitations

- **Unit Variants Only**: The derive macro only supports enums with unit variants (variants without associated data).
- **Attribute Placement**: Ensure that the `#[default_unnormalized_construction_probability = value]` attribute is placed directly above the variant.

## Error Handling

The macro will produce a compile-time error in the following cases:

- If the enum contains non-unit variants.
- If the `default_unnormalized_construction_probability` attribute is used incorrectly.
- If invalid values are provided for the probabilities.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contribution

Contributions are welcome! Please open an issue or submit a pull request on GitHub.

## Acknowledgments

- [syn](https://crates.io/crates/syn), [quote](https://crates.io/crates/quote), and [proc-macro2](https://crates.io/crates/proc-macro2) crates for procedural macro development.
- [rand](https://crates.io/crates/rand) crate for random number generation.
