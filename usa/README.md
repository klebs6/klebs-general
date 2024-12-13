# usa

A Rust library providing typed enums and utilities for working with United States regions, including states, territories, and the federal district. The crate offers:

- Strongly typed representations of U.S. states, territories, and the District of Columbia.
- Simple conversion from names or abbreviations (e.g., `"CA"`, `"California"`, `"Puerto Rico"`, `"DC"`) into typed enums.
- Easy retrieval of official two-letter USPS abbreviations for states, territories, and the District of Columbia.
- Optional serialization and deserialization behavior via [Serde](https://serde.rs/), allowing you to choose between full names or abbreviations through a feature flag.

This crate is useful for applications dealing with geospatial data, address normalization, or any domain where validated U.S. region inputs are required.

## Features

- **`serde_abbreviation`** (requires `serde`): When enabled, serialization and deserialization use the official two-letter abbreviation instead of the full name.

You can enable these features in your `Cargo.toml` as follows:
```toml
[dependencies]
usa = "0.1.0"
```
or, to use abbreviations for serialization:
```toml
[dependencies]
usa = { version = "0.1.0", features = ["serde_abbreviation"] }
```

## Examples

### Parsing Regions from Strings
You can parse states, territories, and the federal district from various string representations:

```rust
use std::convert::TryFrom;
use usa_region::{USRegion, UnitedState, USTerritory, USFederalDistrict};

assert_eq!(USRegion::try_from("CA").unwrap(), USRegion::UnitedState(UnitedState::California));
assert_eq!(USRegion::try_from("Puerto Rico").unwrap(), USRegion::USTerritory(USTerritory::PuertoRico));
assert_eq!(USRegion::try_from("District of Columbia").unwrap(), USRegion::USFederalDistrict(USFederalDistrict::DistrictOfColumbia));

// Case-insensitivity and multiple recognized forms are supported:
assert_eq!("california".parse::UnitedState>().unwrap(), UnitedState::California);
assert_eq!("pr".parse::<USTerritory>().unwrap(), USTerritory::PuertoRico);
assert_eq!("dc".parse::<USFederalDistrict>().unwrap(), USFederalDistrict::DistrictOfColumbia);
```

### Obtaining Abbreviations
Every region implements the `Abbreviation` trait to easily retrieve its official two-letter code:

```rust
use usa_region::{USRegion, UnitedState, USTerritory, USFederalDistrict, Abbreviation};

let california = USRegion::UnitedState(UnitedState::California);
assert_eq!(california.abbreviation(), "CA");

let puerto_rico = USRegion::USTerritory(USTerritory::PuertoRico);
assert_eq!(puerto_rico.abbreviation(), "PR");

let dc = USRegion::USFederalDistrict(USFederalDistrict::DistrictOfColumbia);
assert_eq!(dc.abbreviation(), "DC");
```

### Listing All Regions
You can list all states, territories, or regions at once:

```rust
use usa_region::{UnitedState, USTerritory, USFederalDistrict, USRegion};

// All states:
let all_states = UnitedState::all_states(); 
assert_eq!(all_states.len(), 50);

// All territories:
let all_territories = USTerritory::all_territories();
assert_eq!(all_territories.len(), 5);

// All regions (states + territories + DC):
let all_regions = USRegion::all_regions();
assert_eq!(all_regions.len(), 56); // 50 states + 5 territories + 1 federal district
```

### Serialization and Deserialization with Serde
By default (with `serde` but without `serde_abbreviation`), serialization uses the region's full name. With `serde_abbreviation`, it uses the abbreviation. For example, with `serde_abbreviation` enabled:

```rust
use usa_region::{UnitedState, USTerritory, USRegion};
use serde_json;

let state = UnitedState::California;
let json = serde_json::to_string(&state).unwrap();
assert_eq!(json, "\"CA\""); // With `serde_abbreviation`, abbreviations are used.

// Similarly, for territories and USRegion:
let region = USRegion::USTerritory(USTerritory::Guam);
let json_region = serde_json::to_string(&region).unwrap();
assert_eq!(json_region, "\"GU\"");
```

When deserializing, the crate tries to match either a name or abbreviation. For example:

```rust
let state: UnitedState = serde_json::from_str("\"CA\"").unwrap();
assert_eq!(state, UnitedState::California);

let territory: USTerritory = serde_json::from_str("\"Puerto Rico\"").unwrap();
assert_eq!(territory, USTerritory::PuertoRico); 
```

(Deserialization rules depend on the `serde_abbreviation` feature. Without it, it expects names; with it, it expects abbreviations—but since it supports multiple variants, it will often match regardless.)

## Error Handling
If parsing fails, a `BadInput` error is returned, indicating that the provided string does not correspond to any known region.

```rust
use std::convert::TryFrom;
use usa_region::{USRegion, BadInput};

let result = USRegion::try_from("Narnia");
assert!(result.is_err());
if let Err(e) = result {
    println!("Could not parse region: {}", e); // "Bad input! Narnia"
}
```

## Contributing
Contributions are welcome! Please open an issue or submit a pull request if you have ideas for improvements or additional features.

## License
This project is licensed under the MIT license. See [LICENSE](LICENSE) for details.
