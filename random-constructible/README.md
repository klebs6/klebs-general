random-constructible

**random-constructible** is a Rust crate that provides traits and utilities to easily generate random instances of enums with customizable probabilities. It allows you to:

- Define enums that can be randomly instantiated.
- Assign default weights to enum variants.
- Use custom probability maps to influence the randomness.
- Generate random instances uniformly or based on specified probabilities.

## Features

- **RandConstruct Trait**: A trait that provides methods to generate random instances.
- **RandConstructEnum Trait**: Extends `RandConstruct` for enums, allowing for default weights and custom probability maps.
- **RandConstructProbabilityMapProvider Trait**: Allows for custom probability maps to be provided.
- **Macro for Probability Map Providers**: A convenient macro to define custom probability map providers.
- **Support for Custom Environments**: Use `RandConstructEnvironment` to define environments with specific probability maps.

## Getting Started

### Add Dependency

Add the following to your `Cargo.toml`:

```toml
[dependencies]
random_constructible = "0.1.0"
```

### Derive `RandConstructEnum` for Your Enum

First, define your enum and implement `RandConstructEnum` for it. You'll need to provide:

- A `default_weight` for each variant.
- A list of all variants.
- A default probability map (usually via a provider).

```rust
use random_constructible::{RandConstruct, RandConstructEnum};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum MyEnum {
    VariantA,
    VariantB,
    VariantC,
}

impl Default for MyEnum {
    fn default() -> Self {
        Self::VariantA
    }
}

impl RandConstructEnum for MyEnum {
    fn default_weight(&self) -> f64 {
        match self {
            Self::VariantA => 1.0,
            Self::VariantB => 2.0,
            Self::VariantC => 3.0,
        }
    }

    fn all_variants() -> Vec<Self> {
        vec![Self::VariantA, Self::VariantB, Self::VariantC]
    }

    fn create_default_probability_map() -> Arc<HashMap<Self, f64>> {
        DefaultProvider::probability_map()
    }
}
```

### Define a Probability Map Provider

Use the `random_constructible_probability_map_provider!` macro to define a provider for your enum:

```rust
use random_constructible::random_constructible_probability_map_provider;

struct DefaultProvider;

random_constructible_probability_map_provider!(DefaultProvider => MyEnum {
    VariantA => 1.0,
    VariantB => 2.0,
    VariantC => 3.0,
});
```

### Generate Random Instances

Now you can generate random instances of your enum:

```rust
fn main() {
    let random_variant = MyEnum::random();
    println!("Random Variant: {:?}", random_variant);

    let uniform_variant = MyEnum::uniform();
    println!("Uniform Variant: {:?}", uniform_variant);
}
```

### Use Custom Probability Maps

You can define custom providers to alter the probabilities:

```rust
struct CustomProvider;

random_constructible_probability_map_provider!(CustomProvider => MyEnum {
    VariantA => 5.0,
    VariantB => 1.0,
    VariantC => 1.0,
});

fn main() {
    let custom_random_variant = MyEnum::random_with_provider::<CustomProvider>();
    println!("Custom Random Variant: {:?}", custom_random_variant);
}
```

## Traits and Macros

### `RandConstruct` Trait

Provides basic methods to generate random instances:

- `fn random() -> Self`: Generates a random instance based on default probabilities.
- `fn uniform() -> Self`: Generates a random instance with uniform probability.

### `RandConstructEnum` Trait

Extends `RandConstruct` for enums:

- `fn default_weight(&self) -> f64`: Returns the default weight of a variant.
- `fn all_variants() -> Vec<Self>`: Returns all variants of the enum.
- `fn create_default_probability_map() -> Arc<HashMap<Self, f64>>`: Creates the default probability map.
- Additional methods to sample with custom probabilities and providers.

### `RandConstructProbabilityMapProvider` Trait

Allows custom probability maps:

- `fn probability_map() -> Arc<HashMap<R, f64>>`: Returns the custom probability map.
- `fn uniform_probability_map() -> Arc<HashMap<R, f64>>`: Returns a uniform probability map.

### `random_constructible_probability_map_provider!` Macro

Simplifies the creation of probability map providers:

```rust
random_constructible_probability_map_provider!(ProviderName => EnumType {
    Variant1 => weight1,
    Variant2 => weight2,
    // ...
});
```

## Examples

### Full Example

```rust
use random_constructible::{RandConstruct, RandConstructEnum, random_constructible_probability_map_provider};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum Fruit {
    Apple,
    Banana,
    Cherry,
}

impl Default for Fruit {
    fn default() -> Self {
        Self::Apple
    }
}

impl RandConstructEnum for Fruit {
    fn default_weight(&self) -> f64 {
        match self {
            Self::Apple => 1.0,
            Self::Banana => 1.0,
            Self::Cherry => 1.0,
        }
    }

    fn all_variants() -> Vec<Self> {
        vec![Self::Apple, Self::Banana, Self::Cherry]
    }

    fn create_default_probability_map() -> Arc<HashMap<Self, f64>> {
        DefaultFruitProvider::probability_map()
    }
}

struct DefaultFruitProvider;

random_constructible_probability_map_provider!(DefaultFruitProvider => Fruit {
    Apple => 1.0,
    Banana => 1.0,
    Cherry => 1.0,
});

fn main() {
    let random_fruit = Fruit::random();
    println!("Random Fruit: {:?}", random_fruit);

    let custom_random_fruit = Fruit::random_with_provider::<DefaultFruitProvider>();
    println!("Custom Random Fruit: {:?}", custom_random_fruit);
}
```

### Using a Custom Environment

```rust
use random_constructible::{RandConstructEnum, RandConstructProbabilityMapProvider, RandConstructEnvironment, random_constructible_probability_map_provider};
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

impl RandConstructEnum for Color {
    fn default_weight(&self) -> f64 {
        1.0
    }

    fn all_variants() -> Vec<Self> {
        vec![Self::Red, Self::Green, Self::Blue]
    }

    fn create_default_probability_map() -> Arc<HashMap<Self, f64>> {
        ColorfulEnvironment::probability_map()
    }
}

struct ColorfulEnvironment;

impl RandConstructEnvironment for ColorfulEnvironment {}

random_constructible_probability_map_provider!(ColorfulEnvironment => Color {
    Red => 2.0,
    Green => 3.0,
    Blue => 5.0,
});

fn main() {
    let color = ColorfulEnvironment::create_random::<Color>();
    println!("Random Color from Environment: {:?}", color);
}
```

## Testing

The crate includes a comprehensive set of tests to ensure correctness. The tests cover:

- Validation of all variants.
- Correct default weights.
- Random generation based on default probabilities.
- Uniform random generation.
- Random generation using custom probability maps.
- Sampling using custom providers.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contribution

Contributions are welcome! Please open an issue or submit a pull request on GitHub.

## Acknowledgments

- Inspired by the need for customizable random generation in Rust enums.
- Utilizes the `rand` crate for randomness and `once_cell` for lazy static initialization.

# Contact

For questions or suggestions, feel free to open an issue or contact the maintainer.
