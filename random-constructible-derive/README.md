# random-constructible-derive

[![Crates.io](https://img.shields.io/crates/v/random-constructible-derive.svg)](https://crates.io/crates/random-constructible-derive)
[![Documentation](https://docs.rs/random-constructible-derive/badge.svg)](https://docs.rs/random-constructible-derive)

`random-constructible-derive` is a procedural macro crate that provides custom derive macros for the [`random-constructible`](https://crates.io/crates/random-constructible) crate. It allows you to automatically implement the `RandConstruct` trait for your structs and enums, enabling easy random generation of complex types with minimal boilerplate.

## Features

- **Automatic Implementation**: Derive `RandConstruct` for structs and enums without manual implementation.
- **Support for Enums**: Handle enums with unit variants, named fields, and unnamed fields, including custom probabilities.
- **Support for Structs**: Derive for structs with named, unnamed, or unit fields.
- **Custom Probabilities**: Specify custom probabilities for enum variants and `Option` fields using attributes.
- **Option Field Handling**: Customize the probability of `Some` values in `Option<T>` fields.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
random-constructible = "0.6.0"
random-constructible-derive = "0.6.0"
```

And include it in your crate:

```rust
use random_constructible::RandConstruct;
use random_constructible_derive::RandConstruct;
```

## Usage

### Deriving for Structs

You can derive `RandConstruct` for your structs to enable random generation:

```rust
use random_constructible::RandConstruct;
use random_constructible_derive::RandConstruct;

#[derive(RandConstruct)]
struct MyStruct {
    a: u8,
    b: String,
    c: f64,
}
```

### Deriving for Enums

Similarly, you can derive `RandConstruct` for enums:

```rust
use random_constructible::RandConstruct;
use random_constructible_derive::RandConstruct;

#[derive(RandConstruct)]
enum MyEnum {
    VariantA,
    VariantB(i32, String),
    VariantC { x: f64, y: bool },
}
```

### Custom Probabilities for Enum Variants

You can specify custom probabilities for each variant using the `#[rand_construct(p = ...)]` attribute:

```rust
use random_constructible::RandConstruct;
use random_constructible_derive::RandConstruct;

#[derive(RandConstruct)]
enum MyEnum {
    #[rand_construct(p = 0.5)]
    VariantA,
    #[rand_construct(p = 0.3)]
    VariantB,
    #[rand_construct(p = 0.2)]
    VariantC,
}
```

### Handling `Option` Fields

For `Option<T>` fields, you can specify the probability of generating a `Some` value using the `#[rand_construct(psome = ...)]` attribute:

```rust
use random_constructible::RandConstruct;
use random_constructible_derive::RandConstruct;

#[derive(RandConstruct)]
struct MyStruct {
    #[rand_construct(psome = 0.8)]
    optional_field: Option<String>,
}
```

If you don't specify `psome`, it defaults to `0.5`.

## Examples

### Struct Example

```rust
use random_constructible::RandConstruct;
use random_constructible_derive::RandConstruct;

#[derive(RandConstruct)]
struct Config {
    pub id: u32,
    pub name: String,
    #[rand_construct(psome = 0.9)]
    pub description: Option<String>,
}

fn main() {
    let random_config = Config::random();
    println!("{:?}", random_config);
}
```

### Enum Example with Custom Probabilities

```rust
use random_constructible::RandConstruct;
use random_constructible_derive::RandConstruct;

#[derive(Debug, RandConstruct)]
enum Status {
    #[rand_construct(p = 0.7)]
    Active,
    #[rand_construct(p = 0.2)]
    Inactive,
    #[rand_construct(p = 0.1)]
    Pending,
}

fn main() {
    let random_status = Status::random();
    println!("Random Status: {:?}", random_status);
}
```

### Complex Enum with Fields

```rust
use random_constructible::RandConstruct;
use random_constructible_derive::RandConstruct;

#[derive(Debug, RandConstruct)]
enum Message {
    Text(String),
    Data(u8, u8, u8),
    #[rand_construct(p = 0.05)]
    Disconnect,
}

fn main() {
    let random_message = Message::random();
    println!("Random Message: {:?}", random_message);
}
```

## How It Works

The `random-constructible-derive` crate provides the `RandConstruct` derive macro, which automatically generates implementations of the `RandConstruct` trait for your types. It handles different kinds of data structures:

- **Structs**: Generates random values for each field, respecting any specified attributes.
- **Enums**: Generates random variants based on specified probabilities, and recursively generates random values for any associated data.

### Attributes

- `#[rand_construct(p = <probability>)]`: Sets the probability for an enum variant. The probabilities are relative weights.
- `#[rand_construct(psome = <probability>)]`: Sets the probability of `Some` for `Option<T>` fields.

## Limitations

- The derive macro requires that all fields also implement `RandConstruct`.
- For generic types, ensure that the generic parameters are constrained with `RandConstruct` where necessary.
- The current implementation assumes that probabilities are specified as `f64` literals.

## Contributing

Contributions are welcome! Please feel free to submit a pull request or open an issue for any bugs or feature requests.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

*Note: This crate is a companion to the [`random-constructible`](https://crates.io/crates/random-constructible) crate. Ensure that you have both crates added to your dependencies to utilize the full functionality.*
