# WorldRegion Crate

The **WorldRegion** crate provides a robust, structured representation of countries, continents, and their subregions. It allows you to:

- Convert between `Country` enums and a `WorldRegion` enum that supports continents and subdivided regions (like states/provinces or regional groupings).
- Retrieve ISO3166 Alpha2, Alpha3, and CountryCode representations from world regions.
- Serialize and deserialize `WorldRegion` values to and from JSON, including continent, country, and optional subregion fields.
- Handle errors gracefully through structured error types (e.g., `WorldRegionConversionError`, `WorldRegionParseError`).
- Support abbreviation lookups and conversions to and from `Country`, making it easy to integrate geography into your application.

## Features

- **continent-to-country conversions**: Convert a `Country` into the corresponding `WorldRegion` variant or vice versa.
- **ISO codes**: Extract ISO3166 Alpha2, Alpha3, and `CountryCode` variants directly from a `WorldRegion`.
- **Serialization & Deserialization**: Serialize `WorldRegion` into a structured JSON object with `continent`, `country`, and optional `region` keys. Deserialize back to `WorldRegion` easily.
- **Error Handling**: Fine-grained error enums to understand why a conversion or parse failed.

## Example

```rust
use world_region::{WorldRegion, Country};
use std::convert::TryFrom;

// Convert a known country to its world region
let c = Country::France;
let wr = WorldRegion::try_from(c).expect("France should be in Europe");
println!("{:?}", wr); // e.g., WorldRegion::Europe(EuropeRegion::France(FranceRegion::IleDeFrance))

// Convert back to a country
let back: Country = wr.try_into().expect("Should convert back to France");
assert_eq!(back, Country::France);

// Obtain ISO codes
use std::convert::TryInto;
let alpha2: Iso3166Alpha2 = wr.clone().try_into().expect("Alpha2 conversion");
assert_eq!(alpha2, Iso3166Alpha2::FR);
```

## Serialization / Deserialization

```rust
use serde_json;
use world_region::{WorldRegion, EuropeRegion, FranceRegion};

let wr = WorldRegion::Europe(EuropeRegion::France(FranceRegion::IleDeFrance));
let json = serde_json::to_string(&wr).expect("serialize");
println!("{}", json);
// Outputs: {"continent":"Europe","country":"France","region":"Ile-de-France"}

let deserialized: WorldRegion = serde_json::from_str(&json).expect("deserialize");
assert_eq!(deserialized, wr);
```

## Error Handling

Conversions can fail if a country or region isn't represented. Errors are returned via `Result<T, WorldRegionConversionError>` or `WorldRegionParseError`, allowing you to handle them gracefully.

## License

This crate is distributed under the MIT license. See [LICENSE](LICENSE) for details.
