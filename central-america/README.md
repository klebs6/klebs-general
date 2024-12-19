# Central America Crate

The **central-america** crate provides enums and conversions for Central American and nearby Caribbean countries and their combined region (Haiti and Dominican Republic), analogous to the `asia`, `europe`, and `south-america` crates.

## Features

- **Comprehensive Enum**: `CentralAmericaRegion` includes countries like Belize, Costa Rica, Nicaragua, and a combined region for Haiti and the Dominican Republic.
- **Conversions**: Easily convert `CentralAmericaRegion` to `Country` (from a shared `country` crate), `Iso3166Alpha2`, `Iso3166Alpha3`, and `CountryCode`.
- **Serialization/Deserialization**: Serde support allows serializing and deserializing `CentralAmericaRegion` to/from JSON, including combined regions.
- **Safe and Strict**: No `unsafe`, no `unwrap`, and no `thiserror`. Errors are handled with custom error types.

## Examples

Convert `CentralAmericaRegion` to `Country`:

```rust
use central_america::{CentralAmericaRegion, Country};
use std::convert::TryInto;

let region = CentralAmericaRegion::CostaRica;
let country: Country = region.try_into().expect("Should map to Costa Rica");
assert_eq!(country.to_string(), "Costa Rica");
```

Convert `Country` back to `CentralAmericaRegion`:

```rust
use central_america::{CentralAmericaRegion, Country};
use std::convert::TryInto;

let country = Country::Haiti;
let region: CentralAmericaRegion = country.try_into().expect("Should map to Haiti and Dominican Republic combined region");
assert_eq!(region, CentralAmericaRegion::HaitiAndDominicanRepublic);
```

Serialize and deserialize:

```rust
use central_america::CentralAmericaRegion;
use serde_json;

let region = CentralAmericaRegion::ElSalvador;
let json = serde_json::to_string(&region).expect("serialize");
assert!(json.contains("\"country\":\"El Salvador\""));

let deser: CentralAmericaRegion = serde_json::from_str(&json).expect("deserialize");
assert_eq!(deser, region);
```

## Error Handling

If you attempt to convert a `Country` not represented in Central America to a `CentralAmericaRegion`, you get a `CentralAmericaRegionConversionError`. Similarly, no `unwrap` or `expect` are used internally.

## Contributing

Contributions, issues, and feature requests are welcome! Check the [issues page](https://github.com/yourusername/central-america/issues).

1. Fork it!
2. Create your feature branch: `git checkout -b my-new-feature`
3. Commit changes: `git commit -am 'Add some feature'`
4. Push: `git push origin my-new-feature`
5. Submit a pull request

## License

Licensed under the MIT license. See [LICENSE](LICENSE) for details.
