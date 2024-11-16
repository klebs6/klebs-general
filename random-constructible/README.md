# Random Constructible

[![Crates.io](https://img.shields.io/crates/v/random-constructible.svg)](https://crates.io/crates/random-constructible)
[![Documentation](https://docs.rs/random-constructible/badge.svg)](https://docs.rs/random-constructible)
[![License](https://img.shields.io/crates/l/random-constructible.svg)](https://crates.io/crates/random-constructible)

The `random-constructible` crate provides a trait for creating random instances of enums with weighted probabilities. It's designed to work seamlessly with the companion crate [`random-constructible-derive`](https://crates.io/crates/random-constructible-derive), which provides a procedural macro to automatically implement the trait for your enums.

## Features

- Generate random enum variants based on default or custom probability distributions.
- Support for both uniform and weighted random selection.
- Flexible API allowing the use of custom random number generators (RNGs).
- Ability to define custom probability providers for different contexts or environments.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
random-constructible = "0.1.0"
```

Ensure you also include the `rand` crate if you're not already using it:

```toml
[dependencies]
rand = "0.8"
```

## Usage

First, define your enum and implement the `RandomConstructible` trait. You can do this manually or by using the `random-constructible-derive` crate to automatically generate the implementation (recommended).

### Example with Manual Implementation

```rust
use random_constructible::{RandomConstructible, RandomConstructibleProbabilityMapProvider};
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum Color {
    Red,
    Green,
    Blue,
}

impl Default for Color {
    fn default() -> Self {
        Self::Red
    }
}

impl RandomConstructible for Color {
    fn all_variants() -> Vec<Self> {
        vec![Self::Red, Self::Green, Self::Blue]
    }

    fn default_weight(&self) -> f64 {
        match self {
            Self::Red => 1.0,
            Self::Green => 2.0,
            Self::Blue => 3.0,
        }
    }

    fn default_probability_provider() -> Arc<dyn RandomConstructibleProbabilityMapProvider<Self>> {
        use once_cell::sync::Lazy;

        static DEFAULT_PROVIDER: Lazy<Arc<DefaultProvider>> = Lazy::new(|| Arc::new(DefaultProvider));

        struct DefaultProvider;

        impl RandomConstructibleProbabilityMapProvider<Color> for DefaultProvider {
            fn probability_map(&self) -> Arc<HashMap<Color, f64>> {
                let mut map = HashMap::new();
                map.insert(Color::Red, 1.0);
                map.insert(Color::Green, 2.0);
                map.insert(Color::Blue, 3.0);
                Arc::new(map)
            }
        }

        Arc::clone(&DEFAULT_PROVIDER)
    }
}

fn main() {
    let random_color = Color::random();
    println!("Random Color: {:?}", random_color);
}
```

### Example with `random-constructible-derive` Crate

Using the `random-constructible-derive` crate simplifies the process by automatically generating the required implementations.

```toml
[dependencies]
random-constructible = "0.1.0"
random-constructible-derive = "0.1.0"
rand = "0.8"
```

```rust
use random_constructible::{RandomConstructible, RandomConstructibleProbabilityMapProvider};
use random_constructible_derive::RandomConstructible;
use std::sync::Arc;
use std::collections::HashMap;

#[derive(RandomConstructible, Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum PotionEffect {
    #[default_unnormalized_construction_probability = 5.0]
    Healing,
    #[default_unnormalized_construction_probability = 2.0]
    Poison,
    Strength,
}

impl Default for PotionEffect {
    fn default() -> Self {
        Self::Healing
    }
}

fn main() {
    let random_effect = PotionEffect::random();
    println!("Random Potion Effect: {:?}", random_effect);
}
```

## API Overview

### Trait: `RandomConstructible`

The `RandomConstructible` trait provides methods to generate random instances of an enum.

#### Methods

- `random() -> Self`: Generates a random instance using the default probability distribution.
- `random_with_rng<RNG: Rng + ?Sized>(rng: &mut RNG) -> Self`: Same as `random`, but uses the provided RNG.
- `random_with_probabilities(provider: &dyn RandomConstructibleProbabilityMapProvider<Self>) -> Self`: Generates a random instance using a custom probability provider.
- `random_with_probabilities_rng<RNG: Rng + ?Sized>(provider: &dyn RandomConstructibleProbabilityMapProvider<Self>, rng: &mut RNG) -> Self`: Same as `random_with_probabilities`, but uses the provided RNG.
- `uniform() -> Self`: Generates a random instance with a uniform distribution across all variants.
- `all_variants() -> Vec<Self>`: Returns a vector of all enum variants.
- `default_weight(&self) -> f64`: Returns the default weight for the variant.
- `default_probability_provider() -> Arc<dyn RandomConstructibleProbabilityMapProvider<Self>>`: Returns the default probability provider.

### Trait: `RandomConstructibleProbabilityMapProvider`

Defines a provider for probability maps used in random generation.

#### Method

- `probability_map(&self) -> Arc<HashMap<R, f64>>`: Returns an `Arc` to a `HashMap` containing the probabilities for each variant.

## Custom Probability Providers

You can define custom probability providers to alter the random generation based on different contexts or environments.

```rust
struct ForestEnvironment;

impl RandomConstructibleProbabilityMapProvider<PotionEffect> for ForestEnvironment {
    fn probability_map(&self) -> Arc<HashMap<PotionEffect, f64>> {
        let mut map = HashMap::new();
        map.insert(PotionEffect::Healing, 5.0);
        map.insert(PotionEffect::Poison, 1.0);
        Arc::new(map)
    }
}

fn main() {
    let effect = PotionEffect::random_with_probabilities(&ForestEnvironment);
    println!("Potion Effect in Forest: {:?}", effect);
}
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contribution

Contributions are welcome! Please open an issue or submit a pull request on GitHub.

## Acknowledgments

- [rand](https://crates.io/crates/rand) crate for random number generation.
- [once_cell](https://crates.io/crates/once_cell) crate for lazy static initialization.
