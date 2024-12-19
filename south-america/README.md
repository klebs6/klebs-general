# South America Crate

The **south-america** crate provides enums and conversions for South American countries and their subregions, analogous to the `asia` and `europe` crates. It includes:

- A `SouthAmericaRegion` enum representing South American countries, including subdivided regions for Brazil.
- Conversion traits to convert between `SouthAmericaRegion` and standard ISO country codes.
- Round-trip conversions from `SouthAmericaRegion` to `Country` (from a shared `country` crate) and back.
- Serialization/Deserialization (Serde) support for `SouthAmericaRegion` and its subregions (like `BrazilRegion`).
- Comprehensive test suites ensuring correctness and stability.

## Features

- **No unsafe code**: Entirely safe Rust.
- **No unwrap/expect**: Error handling via `Result` and custom error types.
- **No thiserror**: Manual error handling for full control.

## Examples

Convert a `SouthAmericaRegion` to a `Country`:

```rust
use south_america::{SouthAmericaRegion, Country};
use std::convert::TryInto;

let region = SouthAmericaRegion::Brazil(south_america::BrazilRegion::Sul);
let country: Country = region.try_into().expect("Should map to Brazil");
assert_eq!(country.to_string(), "Brazil");
```

Convert a `Country` back to `SouthAmericaRegion`:

```rust
use south_america::{SouthAmericaRegion, Country};
use std::convert::TryInto;

let country = Country::Argentina;
let region: SouthAmericaRegion = country.try_into().expect("Should map to Argentina");
assert_eq!(region, SouthAmericaRegion::Argentina);
```

Serialize and deserialize `SouthAmericaRegion` with JSON:

```rust
use south_america::{SouthAmericaRegion, BrazilRegion};
use serde_json;

let region = SouthAmericaRegion::Brazil(BrazilRegion::Nordeste);
let json = serde_json::to_string(&region).expect("serialize");
assert!(json.contains("\"country\":\"Brazil\""));
assert!(json.contains("\"region\":\"Nordeste\""));

let deser: SouthAmericaRegion = serde_json::from_str(&json).expect("deserialize");
assert_eq!(deser, region);
```

## Error Handling

If attempting impossible conversions (e.g., non-South American countries to `SouthAmericaRegion`), an `SouthAmericaRegionConversionError` is returned.

## Contributing

Contributions, issues, and feature requests are welcome! Check the [issues page](https://github.com/yourusername/south-america/issues).

1. Fork it!
2. Create your feature branch: `git checkout -b my-new-feature`
3. Commit changes: `git commit -am 'Add some feature'`
4. Push: `git push origin my-new-feature`
5. Submit a pull request

## License

Licensed under the MIT license. See [LICENSE](LICENSE) for details.
