# North America Crate

The **north-america** crate provides enums and conversions for North American countries and their subregions, analogous to the `asia`, `europe`, `south-america`, and `central-america` crates. It includes:

- A `NorthAmericaRegion` enum with:
  - `Canada(CanadaRegion)`
  - `Greenland`
  - `Mexico`
  - `UnitedStates(USRegion)`, leveraging the existing `usa` crate for U.S. subdivisions.
- Conversions between `NorthAmericaRegion` and `Country`.
- ISO code conversions (`Iso3166Alpha2`, `Iso3166Alpha3`, `CountryCode`).
- Serde support for serialization/deserialization, including subdivided regions.
- No unsafe code, no unwrap, and no thiserror. All error handling is manual and explicit.

## Examples

Convert `NorthAmericaRegion` to `Country`:

```rust
use north_america::{NorthAmericaRegion, CanadaRegion, Country};
use std::convert::TryInto;

let region = NorthAmericaRegion::Canada(CanadaRegion::Ontario);
let country: Country = region.try_into().expect("Should map to Canada");
assert_eq!(country.to_string(), "Canada");
```

Convert `Country` back to `NorthAmericaRegion`:

```rust
use north_america::{NorthAmericaRegion, Country};
use std::convert::TryInto;

let country = Country::Mexico;
let region: NorthAmericaRegion = country.try_into().expect("Should map to Mexico");
assert_eq!(region, NorthAmericaRegion::Mexico);
```

Serialize and deserialize:

```rust
use north_america::{NorthAmericaRegion, CanadaRegion};
use serde_json;

let region = NorthAmericaRegion::Canada(CanadaRegion::BritishColumbia);
let json = serde_json::to_string(&region).expect("serialize");
assert!(json.contains("\"country\":\"Canada\""));
assert!(json.contains("\"region\":\"British Columbia\""));

let deser: NorthAmericaRegion = serde_json::from_str(&json).expect("deserialize");
assert_eq!(deser, region);
```

## Error Handling

Non-North American countries fail conversion with a `NorthAmericaRegionConversionError`.

## Contributing

Issues and PRs are welcome.

## License

MIT licensed. See [LICENSE](LICENSE).
