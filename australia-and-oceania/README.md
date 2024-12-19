# australia-oceania-antarctica Crate

The `australia-oceania-antarctica` crate provides an enumeration of countries and special regions in the Australia/Oceania/Antarctica area, analogous to the `africa` crate.

## Features

- **Comprehensive Enumeration:** 
  Includes Australia, numerous island states in Oceania, and Antarctica, as well as special territories like French Polynesia and Niue.

- **Conversions to/from `Country`:** 
  Converts region variants to `Country` where possible. Returns detailed errors for unsupported territories or combined regions.

- **ISO Codes and Abbreviations:** 
  Each variant can produce abbreviations, and when converting to `Country` you can obtain ISO codes (if a direct mapping exists).

- **Serialization/Deserialization:** 
  Supports `serde` for easy serialization to and from JSON:
  ```json
  { "country": "Samoa" }
  ```
  deserializes back into `AustraliaOceaniaAntarcticaRegion::Samoa`.

- **Error Handling:** 
  Employs robust error handling returning typed errors (`AoaRegionConversionError`).

## Example

```rust
use australia_oceania_antarctica::AustraliaOceaniaAntarcticaRegion;
use std::convert::TryInto;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let region = AustraliaOceaniaAntarcticaRegion::NewZealand;
    let country = region.try_into()?; // Convert to Country
    println!("Country: {:?}", country);

    let json = serde_json::to_string(&region)?;
    println!("Serialized: {}", json);

    let deserialized: AustraliaOceaniaAntarcticaRegion = serde_json::from_str(&json)?;
    println!("Deserialized: {:?}", deserialized);

    Ok(())
}
```

## Testing

Run the tests with:

```bash
cargo test
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
