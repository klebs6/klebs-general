# rand-construct

`rand-construct` is a Rust crate that provides a derive macro to automatically implement traits for generating random instances of your types. It allows you to easily generate random enums and structs, with customizable probabilities for enum variants, and supports custom environments to influence randomness.

## Features

- **Automatic Trait Implementation**: Derive `RandConstruct` for your enums and structs.
- **Customizable Probabilities**: Specify default unnormalized probabilities for enum variants using attributes.
- **Custom Environments**: Create environments to alter probabilities for random generation.
- **Easy Integration**: Seamlessly integrates with the `rand` crate for randomness.

## Getting Started

### Add Dependency

Add the following to your `Cargo.toml`:

```toml
[dependencies]
rand-construct = "0.6.0"  # Replace with the latest version
```

### Import the Crate

In your Rust file, import the necessary traits and macros:

```rust
use rand_construct::{RandConstruct, RandConstructEnvironment};
```

## Usage

### Deriving `RandConstruct` for Enums

You can automatically implement random construction for your enums by using the `#[derive(RandConstruct)]` macro. You can also specify default unnormalized probabilities for each variant using the `#[rand_construct(p = value)]` attribute.

```rust
use rand_construct::RandConstruct;

#[derive(RandConstruct, Debug)]
enum Weather {
    #[rand_construct(p = 5.0)]
    Sunny,
    #[rand_construct(p = 2.0)]
    Rainy,
    #[rand_construct(p = 1.0)]
    Cloudy,
    #[rand_construct(p = 0.5)]
    Snowy,
}

fn main() {
    let random_weather = Weather::random();
    println!("Random Weather: {:?}", random_weather);
}
```

#### Explanation

- `#[derive(RandConstruct)]`: Automatically implements random construction for `Weather`.
- `#[rand_construct(p = value)]`: Sets the default weight for each variant.
- `Weather::random()`: Generates a random instance of `Weather` based on the specified probabilities.

### Deriving `RandConstruct` for Structs

For structs, you can derive `RandConstruct`, and the macro will automatically implement the trait by generating random instances of each field.

```rust
use rand_construct::RandConstruct;

#[derive(RandConstruct, Debug)]
struct Dimensions {
    width: u32,
    height: u32,
}

fn main() {
    let random_dimensions = Dimensions::random();
    println!("Random Dimensions: {:?}", random_dimensions);
}
```

#### Note

- All fields in the struct must implement `RandConstruct`.
- For primitive types like `u32`, `RandConstruct` is implemented by default.

### Creating and Using Environments

You can define custom environments to alter the probabilities of random generation. This is useful when you want different contexts to influence the randomness.

#### Defining an Environment

First, derive `RandConstructEnvironment` for your environment struct:

```rust
use rand_construct::RandConstructEnvironment;

#[derive(RandConstructEnvironment)]
struct DesertEnvironment;
```

Then, use the `rand_construct_env!` macro to define custom probabilities for your enums within this environment:

```rust
use rand_construct::{rand_construct_env, RandConstruct};

#[derive(RandConstruct, Debug)]
enum Animal {
    #[rand_construct(p = 1.0)]
    Camel,
    #[rand_construct(p = 1.0)]
    Scorpion,
    #[rand_construct(p = 1.0)]
    FennecFox,
}

rand_construct_env! {
    DesertEnvironment => Animal {
        Camel      => 5.0,
        Scorpion   => 3.0,
        FennecFox  => 2.0,
    }
}
```

#### Using the Environment

You can now generate random instances using the custom environment:

```rust
fn main() {
    let desert_animal = Animal::random_with_env::<DesertEnvironment>();
    println!("Desert Animal: {:?}", desert_animal);
}
```

#### Explanation

- `#[derive(RandConstructEnvironment)]`: Marks `DesertEnvironment` as a custom environment.
- `rand_construct_env!`: Defines custom probabilities for `Animal` variants within `DesertEnvironment`.
- `Animal::random_with_env::<DesertEnvironment>()`: Generates a random `Animal` using the probabilities defined in `DesertEnvironment`.

### Full Example with Environment

```rust
use rand_construct::{RandConstruct, RandConstructEnvironment, rand_construct_env};

#[derive(RandConstruct, Debug)]
enum Terrain {
    #[rand_construct(p = 1.0)]
    Sand,
    #[rand_construct(p = 1.0)]
    Rock,
    #[rand_construct(p = 1.0)]
    Oasis,
}

rand_construct_env! {
    DesertEnvironment => Terrain {
        Sand   => 8.0,
        Rock   => 2.0,
        Oasis  => 1.0,
    }
}

fn main() {
    let random_terrain = Terrain::random_with_env::<DesertEnvironment>();
    println!("Random Terrain in Desert: {:?}", random_terrain);
}
```

#### Output

```
Random Terrain in Desert: Sand
```

## Attributes

### `#[rand_construct(p = value)]`

- **Applies to**: Enum variants.
- **Purpose**: Sets the default weight (unnormalized probability) for the variant.
- **Default**: If not specified, the default weight is `1.0`.

#### Example

```rust
#[derive(RandConstruct, Debug)]
enum Beverage {
    #[rand_construct(p = 3.0)]
    Water,
    #[rand_construct(p = 1.0)]
    Soda,
    #[rand_construct(p = 0.5)]
    Juice,
    Coffee, // Default weight is 1.0
}
```

## Advanced Usage

### Generating Uniform Random Instances

You can generate random instances with uniform probabilities using the `uniform` method:

```rust
use rand_construct::RandConstruct;

fn main() {
    let uniform_weather = Weather::uniform();
    println!("Uniform Weather: {:?}", uniform_weather);
}
```

### Using a Custom Random Number Generator

You can generate random instances using a custom random number generator:

```rust
use rand::rngs::StdRng;
use rand::SeedableRng;
use rand_construct::RandConstruct;

fn main() {
    let mut rng = StdRng::seed_from_u64(42);
    let random_weather = Weather::random_with_rng(&mut rng);
    println!("Random Weather with Custom RNG: {:?}", random_weather);
}
```

## Limitations

- **Enums with Non-Unit Variants**: The derive macro for enums currently supports only unit variants (variants without associated data).
- **Field Types**: All fields in structs must implement `RandConstruct`.

## Integration with the `rand` Crate

`rand-construct` integrates seamlessly with the [`rand`](https://crates.io/crates/rand) crate. It uses `rand` under the hood to generate random values.

## License

This project is licensed under the Apache 2.0 License. See the [LICENSE](LICENSE) file for details.

## Contribution

Contributions are welcome! Please open an issue or submit a pull request on GitHub.

## Acknowledgments

- Inspired by the need to simplify random instance generation for Rust types.
- Utilizes procedural macros to reduce boilerplate.
