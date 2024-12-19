# Africa Crate

The `africa` crate provides an enumeration of African regions (primarily countries and a few combined or special territories) analogous to the `asia` crate in the provided example. It supports serialization, deserialization, conversion to and from `Country` enums, ISO codes, and also provides abbreviations for each region.

## Features

- **Robust Enumeration of African Regions:** 
  All African countries are represented as variants of `AfricaRegion`, along with certain combined or special regions (e.g., Canary Islands, Senegal and Gambia, Saint Helena, Ascension, and Tristan da Cunha).

- **Conversions to and from `Country`:** 
  The crate allows converting from `AfricaRegion` to `Country` where possible, and from `Country` to `AfricaRegion` for African countries. Combined or non-mappable regions return informative errors.

- **ISO Codes and Abbreviations:** 
  Each `AfricaRegion` variant can be converted to `Iso3166Alpha2`, `Iso3166Alpha3`, and `CountryCode` when applicable. Additionally, the `Abbreviation` trait provides short codes for each region (for example, `Nigeria` -> "NG").

- **Serialization/Deserialization:** 
  Out-of-the-box `serde` support allows you to serialize and deserialize `AfricaRegion` values to JSON. By default, regions are stored as:
  ```json
  {
    "country": "Nigeria"
  }
  ```
  You can deserialize these structures back into `AfricaRegion` variants.

- **Error Handling:** 
  Instead of panicking, all conversions that may fail return strongly typed error variants (`AfricaRegionConversionError`), enabling robust error handling in production systems.

## Example

```rust
use africa::AfricaRegion;
use std::convert::TryInto;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let region = AfricaRegion::Nigeria;
    let country = region.try_into()?; // Convert to Country
    println!("Country: {:?}", country);

    let iso_code: country::Iso3166Alpha2 = region.try_into()?;
    println!("ISO Alpha-2: {:?}", iso_code);

    let json = serde_json::to_string(&region)?;
    println!("Serialized: {}", json);

    let deserialized: AfricaRegion = serde_json::from_str(&json)?;
    println!("Deserialized: {:?}", deserialized);

    Ok(())
}
```

## Testing

A comprehensive test suite is provided. Simply run:

```bash
cargo test
```

This will run all tests ensuring correctness of default values, conversions, abbreviations, serialization/deserialization, and error handling.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
