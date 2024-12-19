# europe

**europe** is a Rust crate for enumerating and working with European countries and their subdivisions. It provides a comprehensive set of enums representing European countries, as well as nested enums for various regions, provinces, and federations within those countries. The crate supports serialization and deserialization with `serde`, enabling seamless integration with JSON and other formats, while preserving region-specific details.

## Features

- **Comprehensive Geographical Coverage:**  
  Includes a wide range of European countries and their respective regions (like France’s regions, Germany’s Bundesländer, Italy’s regions, etc.).

- **Nested Regions:**  
  Subdivided countries are represented as enum variants containing region-level enums (e.g., `EuropeRegion::France(FranceRegion::Bretagne)`).

- **Serialization/Deserialization:**  
  Uses `serde` for serialization and deserialization. In non-abbreviation mode, subdivided countries are serialized as structured maps (`{ "country": "France", "region": "Bretagne" }`), ensuring lossless round-trips of region data.

- **Abbreviations (Optional):**  
  With the `serde_abbreviation` feature, regions are serialized to their abbreviation codes (e.g., `FR` for France, `GB` for the UK, `DE` for Germany’s regions, etc.).

- **Reliable Parsing:**  
  Invalid input during deserialization leads to descriptive `DeError::unknown_variant` errors. Default values are defined for subdivided regions, ensuring that if a region field is missing, the default region is used.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
europe = "0.1"
```

Then in your code:

```rust
use europe::{EuropeRegion, FranceRegion};
use serde_json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let region = EuropeRegion::France(FranceRegion::Bretagne);
    let serialized = serde_json::to_string(&region)?;
    println!("Serialized: {}", serialized);

    let deserialized: EuropeRegion = serde_json::from_str(&serialized)?;
    assert_eq!(deserialized, region);
    println!("Deserialized matches original!");

    Ok(())
}
```

## Testing

The crate includes an extensive set of tests covering:

- Default values
- Round-trip serialization/deserialization
- Parsing from strings
- Handling unknown variants
- Ensuring that subdivided countries retain their exact region variants after round-tripping

Run the tests using:

```bash
cargo test
```

All tests should pass.

## License

This project is licensed under the MIT license (or your chosen license). See [LICENSE](./LICENSE) for details.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request for any improvements or bug fixes.
