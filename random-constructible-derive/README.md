# RandomConstructible-Derive

**RandomConstructible-Derive** is a Rust procedural macro crate that provides custom derives to automatically implement the `RandomConstructible` and `RandomConstructibleEnum` traits from the [RandomConstructible](https://crates.io/crates/random_constructible) crate. This allows you to easily generate random instances of your structs and enums without manually implementing the necessary traits.

## Features

- **Automatic Trait Implementation**: Derive `RandomConstructible` and `RandomConstructibleEnum` for your structs and enums.
- **Customizable Probabilities**: Specify default unnormalized construction probabilities for enum variants using attributes.
- **Support for Environments**: Derive `RandomConstructibleEnvironment` for your custom environments.
- **Easy Integration**: Seamlessly integrates with the `RandomConstructible` crate for a smooth development experience.

## Getting Started

### Add Dependencies

Add the following to your `Cargo.toml`:

```toml
[dependencies]
random-constructible = "0.1.0"
random-constructible-derive = "0.1.0"
```

Ensure that you include both the `random-constructible` and `random-constructible-derive` crates.

### Import the Crate

In your Rust file, import the necessary traits and macros:

```rust
use random_constructible::{RandomConstructible, RandomConstructibleEnum};
use random_constructible_derive::RandomConstructible;
```

## Usage

### Deriving `RandomConstructible` for Enums

You can automatically implement `RandomConstructibleEnum` for your enums by using the `#[derive(RandomConstructible)]` macro. You can also specify default unnormalized construction probabilities for each variant using the `#[default_unnormalized_construction_probability = "value"]` attribute.

```rust
use random_constructible::{RandomConstructible, RandomConstructibleEnum};
use random_constructible_derive::RandomConstructible;

#[derive(RandomConstructible, Debug)]
enum MyEnum {
    #[default_unnormalized_construction_probability = "2.0"]
    VariantA,
    #[default_unnormalized_construction_probability = "3.0"]
    VariantB,
    #[default_unnormalized_construction_probability = "5.0"]
    VariantC,
}

fn main() {
    let random_variant = MyEnum::random();
    println!("Random Variant: {:?}", random_variant);
}
```

#### Explanation

- `#[derive(RandomConstructible)]`: Automatically implements `RandomConstructibleEnum` for `MyEnum`.
- `#[default_unnormalized_construction_probability = "value"]`: Sets the default weight for each variant.

### Deriving `RandomConstructible` for Structs

For structs, you can derive `RandomConstructible`, and the macro will automatically implement the trait by generating random instances of each field.

```rust
use random_constructible::RandomConstructible;
use random_constructible_derive::RandomConstructible;

#[derive(RandomConstructible, Debug)]
struct MyStruct {
    x: i32,
    y: f64,
}

fn main() {
    let random_struct = MyStruct::random();
    println!("Random Struct: {:?}", random_struct);
}
```

#### Note

- All fields in the struct must implement `RandomConstructible`.
- For primitive types, you may need to implement `RandomConstructible` or use existing implementations.

### Deriving `RandomConstructibleEnvironment`

You can also derive `RandomConstructibleEnvironment` for your custom environments:

```rust
use random_constructible::{RandomConstructibleEnvironment, RandomConstructibleProbabilityMapProvider};
use random_constructible_derive::RandomConstructibleEnvironment;

#[derive(RandomConstructibleEnvironment)]
struct MyEnvironment;

fn main() {
    // Use your environment to create random instances
}
```

## Examples

### Full Enum Example

```rust
use random_constructible::{RandomConstructible, RandomConstructibleEnum};
use random_constructible_derive::RandomConstructible;
use std::fmt;

#[derive(RandomConstructible, Debug)]
enum Color {
    #[default_unnormalized_construction_probability = "1.0"]
    Red,
    #[default_unnormalized_construction_probability = "2.0"]
    Green,
    #[default_unnormalized_construction_probability = "3.0"]
    Blue,
}

fn main() {
    let random_color = Color::random();
    println!("Random Color: {:?}", random_color);

    let uniform_color = Color::uniform();
    println!("Uniform Color: {:?}", uniform_color);
}
```

#### Output

```
Random Color: Blue
Uniform Color: Green
```

### Full Struct Example

```rust
use random_constructible::RandomConstructible;
use random_constructible_derive::RandomConstructible;

#[derive(RandomConstructible, Debug)]
struct Point {
    x: i32,
    y: i32,
}

impl RandomConstructible for i32 {
    fn random() -> Self {
        rand::random::<i32>()
    }

    fn uniform() -> Self {
        rand::random::<i32>()
    }
}

fn main() {
    let random_point = Point::random();
    println!("Random Point: {:?}", random_point);
}
```

#### Output

```
Random Point: Point { x: 42, y: -17 }
```

### Custom Probability Map Provider with Environment

```rust
use random_constructible::{
    random_constructible_probability_map_provider, RandomConstructible,
    RandomConstructibleEnum, RandomConstructibleEnvironment,
};
use random_constructible_derive::{RandomConstructible, RandomConstructibleEnvironment};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(RandomConstructible, Debug)]
enum Fruit {
    #[default_unnormalized_construction_probability = "1.0"]
    Apple,
    #[default_unnormalized_construction_probability = "1.0"]
    Banana,
    #[default_unnormalized_construction_probability = "8.0"]
    Cherry,
}

#[derive(RandomConstructibleEnvironment)]
struct FruitEnvironment;

random_constructible_probability_map_provider!(FruitEnvironment => Fruit {
    Apple => 1.0,
    Banana => 1.0,
    Cherry => 8.0,
});

fn main() {
    let random_fruit = FruitEnvironment::create_random::<Fruit>();
    println!("Random Fruit from Environment: {:?}", random_fruit);
}
```

#### Output

```
Random Fruit from Environment: Cherry
```

## Attributes

### `#[default_unnormalized_construction_probability = "value"]`

- **Applies to**: Enum variants.
- **Purpose**: Sets the default weight (unnormalized probability) for the variant.
- **Default**: If not specified, the default weight is `1.0`.

#### Example

```rust
#[derive(RandomConstructible)]
enum Vehicle {
    #[default_unnormalized_construction_probability = "5.0"]
    Car,
    #[default_unnormalized_construction_probability = "2.0"]
    Bike,
    Truck, // Default weight is 1.0
}
```

## How It Works

### Enums

When you derive `RandomConstructible` for an enum:

- The macro implements `RandomConstructibleEnum` for the enum.
- It generates:

  - A method to return all variants.
  - A method to get the default weight for each variant.
  - A method to create a default probability map using `once_cell` for lazy initialization.

### Structs

When you derive `RandomConstructible` for a struct:

- The macro implements `RandomConstructible` for the struct.
- It recursively generates random instances of each field.
- Requires all fields to implement `RandomConstructible`.

### Environments

When you derive `RandomConstructibleEnvironment`:

- The macro implements `RandomConstructibleEnvironment` for the struct.
- Allows you to define custom environments with specific probability maps.

## Limitations

- **Enums with Non-Unit Variants**: The derive macro for enums only supports unit variants (variants without associated data).
- **Unions**: The derive macros do not support unions.
- **Field Types**: All fields in structs must implement `RandomConstructible`.

## Integration with `RandomConstructible` Crate

Ensure that you have the `random-constructible` crate in your dependencies and that you import the necessary traits:

```rust
use random_constructible::{RandomConstructible, RandomConstructibleEnum};
```

## Advanced Usage

### Custom Implementations

If you need more control, you can manually implement `RandomConstructibleEnum` or `RandomConstructible` for your types.

### Using Custom Rng

You can generate random instances using a custom random number generator:

```rust
use rand::rngs::StdRng;
use rand::SeedableRng;

fn main() {
    let mut rng = StdRng::seed_from_u64(42);
    let random_variant = MyEnum::random_with_rng(&mut rng);
    println!("Random Variant with Custom Rng: {:?}", random_variant);
}
```

## Testing

The `random-constructible-derive` crate should be tested in conjunction with the `random-constructible` crate to ensure that the derives work as expected.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contribution

Contributions are welcome! Please open an issue or submit a pull request on GitHub.

## Acknowledgments

- Inspired by the need to simplify random instance generation for Rust types.
- Utilizes the `proc-macro` crate for procedural macros and `syn` and `quote` for parsing and generating Rust code.

# Contact

For questions or suggestions, feel free to open an issue or contact the maintainer.

---

Happy coding!
