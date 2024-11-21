# random-constructible

[![Crates.io](https://img.shields.io/crates/v/random-constructible.svg)](https://crates.io/crates/random-constructible)
[![Documentation](https://docs.rs/random-constructible/badge.svg)](https://docs.rs/random-constructible)

`random-constructible` is a Rust crate that provides traits and macros to facilitate the random generation of primitive types and enums, with support for custom probability distributions. It simplifies the process of creating random instances of your types, especially when dealing with enums that require weighted random selection.

## Features

- **Random Generation for Primitive Types**: Automatically implements random generation for all primitive integer and floating-point types.
- **Random Enums with Custom Probabilities**: Easily define how your enums should be randomly generated, including specifying custom weights for each variant.
- **Uniform Random Generation**: Support for uniform random generation across all variants.
- **Extensible Probability Maps**: Create and use custom probability maps for more complex random generation scenarios.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
random-constructible = "0.6.0"
```

and include it in your crate:

```rust
use random_constructible::*;
```

## Getting Started

### Random Generation for Primitive Types

The crate automatically provides implementations for all primitive integer and floating-point types. You can generate random values using:

```rust
use random_constructible::RandConstruct;

let random_u32: u32 = u32::random();
let random_f64: f64 = f64::random();
```

### Random Enums

To enable random generation for your enums, implement the `RandConstructEnum` trait. You can do this manually or use the provided macros for convenience.

#### Manual Implementation

```rust
use random_constructible::{RandConstructEnum, RandConstruct};

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum MyEnum {
    #[default]
    VariantA,
    VariantB,
    VariantC,
}

impl RandConstructEnum for MyEnum {
    fn all_variants() -> Vec<Self> {
        vec![Self::VariantA, Self::VariantB, Self::VariantC]
    }

    fn default_weight(&self) -> f64 {
        match self {
            Self::VariantA => 1.0,
            Self::VariantB => 2.0,
            Self::VariantC => 3.0,
        }
    }

    fn create_default_probability_map() -> std::sync::Arc<std::collections::HashMap<Self, f64>> {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        for variant in Self::all_variants() {
            map.insert(variant, variant.default_weight());
        }
        std::sync::Arc::new(map)
    }
}
```

#### Using the `rand_construct_env!` Macro

Alternatively, you can use the `rand_construct_env!` macro to define the probability map:

```rust
use random_constructible::{RandConstructEnumWithEnv, rand_construct_env};

struct DefaultProvider;

rand_construct_env!(DefaultProvider => MyEnum {
    VariantA => 1.0,
    VariantB => 2.0,
    VariantC => 3.0,
});
```

### Sampling Random Variants

Once you've implemented `RandConstructEnum` for your enum, you can generate random variants:

```rust
use random_constructible::RandConstruct;

let random_variant = MyEnum::random(); // Uses default weights
let uniform_variant = MyEnum::uniform(); // Uniform probability
```

To sample using a custom provider:

```rust
use random_constructible::RandConstructEnumWithEnv;

let random_variant = MyEnum::sample_from_provider::<DefaultProvider, _>(&mut rand::thread_rng());
```

## Examples

### Complete Example

```rust
use random_constructible::{RandConstructEnum, RandConstructEnumWithEnv, rand_construct_env, RandConstruct};
use rand::Rng;

// Define your enum
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum Color {
    #[default]
    Red,
    Green,
    Blue,
}

// Implement RandConstructEnum
impl RandConstructEnum for Color {
    fn all_variants() -> Vec<Self> {
        vec![Self::Red, Self::Green, Self::Blue]
    }

    fn default_weight(&self) -> f64 {
        match self {
            Self::Red => 1.0,
            Self::Green => 1.0,
            Self::Blue => 1.0,
        }
    }

    fn create_default_probability_map() -> std::sync::Arc<std::collections::HashMap<Self, f64>> {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        for variant in Self::all_variants() {
            map.insert(variant, variant.default_weight());
        }
        std::sync::Arc::new(map)
    }
}

// Define a custom probability provider
struct CustomColorProvider;

rand_construct_env!(CustomColorProvider => Color {
    Red => 0.5,
    Green => 0.3,
    Blue => 0.2,
});

fn main() {
    // Random variant using default weights
    let random_color = Color::random();
    println!("Random Color: {:?}", random_color);

    // Random variant using custom probabilities
    let mut rng = rand::thread_rng();
    let random_color = Color::sample_from_provider::<CustomColorProvider, _>(&mut rng);
    println!("Custom Random Color: {:?}", random_color);
}
```

### Testing Randomness

The crate also provides utilities for testing the distribution of your random generation:

```rust
use random_constructible::{RandConstructEnum, RandConstruct};
use std::collections::HashMap;

fn main() {
    let mut counts = HashMap::new();
    for _ in 0..10000 {
        let variant = Color::random();
        *counts.entry(variant).or_insert(0) += 1;
    }

    for (variant, count) in counts {
        println!("{:?}: {}", variant, count);
    }
}
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

*Note: This README is generated based on the crate's code and is meant to help you get started with `random-constructible`. For more detailed information, please refer to the [documentation](https://docs.rs/random-constructible).*
