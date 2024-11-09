# Lyrical Meter

[![Crates.io](https://img.shields.io/crates/v/lyrical-meter.svg)](https://crates.io/crates/lyrical-meter)
[![Documentation](https://docs.rs/lyrical-meter/badge.svg)](https://docs.rs/lyrical-meter)
[![License](https://img.shields.io/crates/l/lyrical-meter.svg)](https://github.com/yourusername/lyrical-meter/blob/master/LICENSE)

A Rust crate for representing and working with various poetic meters. Ideal for applications in poetry generation, analysis, and education.

## Features

- **Standard Meters**: Represent standard metrical feet like iamb, trochee, anapest, etc.
- **Line Lengths**: Handle line lengths from monometer to decameter.
- **Non-Standard Meters**: Include non-standard meters like free verse and mixed meter.
- **AI-Friendly Descriptions**: Generate descriptions suitable for natural language processing tasks.
- **Random Generation**: Generate random meters for testing or generative applications.
- **Builder Patterns**: Flexible and fluent construction of meters using builder patterns.
- **Serialization Support**: Serialize and deserialize meters using `serde`.
- **Trait Implementations**: Standard traits like `Clone`, `Copy`, `Debug`, `PartialEq`, and `Eq`.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
lyrical-meter = "0.1.0"
```

Then, in your Rust code:

```rust
extern crate lyrical_meter;
```

## Usage

### Creating a Standard Meter

```rust
use lyrical_meter::{Meter, LyricalMeter, MetricalFoot, LineLength};

fn main() {
    let meter = Meter::Standard(
        LyricalMeter::builder()
            .foot(MetricalFoot::Iamb)
            .length(LineLength::Pentameter)
            .build(),
    );

    println!("Meter: {}", meter);
    println!("AI Description: {}", meter.ai());
}
```

**Output:**

```
Meter: Iamb in Pentameter
AI Description: Use iambic meter, with unstressed-stressed syllables. Each line should have five feet (pentameter).
```

### Creating an Other Meter

```rust
use lyrical_meter::{Meter, OtherMeter};

fn main() {
    let meter = Meter::Other(OtherMeter::FreeVerse);

    println!("Meter: {}", meter);
    println!("AI Description: {}", meter.ai());
}
```

**Output:**

```
Meter: Write in free verse, without a consistent meter or rhyme scheme.
AI Description: Write in free verse, without a consistent meter or rhyme scheme.
```

### Random Meter Generation

```rust
use lyrical_meter::Meter;
use rand::Rng;

fn main() {
    let mut rng = rand::thread_rng();
    let random_meter: Meter = rng.gen();

    println!("Random Meter: {}", random_meter);
    println!("AI Description: {}", random_meter.ai());
}
```

### Serialization and Deserialization

```rust
use lyrical_meter::{Meter, OtherMeter};
use serde_json;

fn main() {
    let meter = Meter::Other(OtherMeter::BlankVerse);
    let serialized = serde_json::to_string(&meter).unwrap();
    let deserialized: Meter = serde_json::from_str(&serialized).unwrap();

    assert_eq!(meter, deserialized);
    println!("Serialized Meter: {}", serialized);
}
```

**Output:**

```
Serialized Meter: {"Other":"BlankVerse"}
```

### Using Builders and Setters

```rust
use lyrical_meter::{Meter, LyricalMeter, MetricalFoot, LineLength};

fn main() {
    // Using the builder pattern
    let mut lyrical_meter = LyricalMeter::builder()
        .foot(MetricalFoot::Trochee)
        .length(LineLength::Tetrameter)
        .build();

    // Modifying using setters
    lyrical_meter.set_foot(MetricalFoot::Anapest);
    lyrical_meter.set_length(Some(LineLength::Trimeter));

    println!("Modified Meter: {}", lyrical_meter);
}
```

**Output:**

```
Modified Meter: Anapest in Trimeter
```

## Documentation

For more detailed information, please refer to the [documentation](https://docs.rs/lyrical-meter).

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contribution

Contributions are welcome! Please open an issue or submit a pull request on [GitHub](https://github.com/yourusername/lyrical-meter).

## Acknowledgments

Inspired by the need for a robust and flexible representation of poetic meters in Rust applications.
