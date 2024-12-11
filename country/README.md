# country

`country` is a Rust library providing a single source of truth for country enumeration and their associated ISO 3166-1 alpha-2 and alpha-3 codes. By using a macro-based approach, all country variants, along with their standardized ISO codes, are defined and linked together in one place, eliminating redundancy and reducing maintenance overhead.

## Features

- **Single Source of Truth:**  
  All `Country` variants and their corresponding ISO codes (`Iso3166Alpha2`, `Iso3166Alpha3`) are declared once via a macro. This generates:
  - A `Country` enum enumerating countries.
  - `Iso3166Alpha2` and `Iso3166Alpha3` enums representing standardized country codes.
  - Bidirectional conversions between `Country` and ISO codes.
  
- **Automatic Code Generation:**  
  The macro expands into:
  - `From<Country>` conversions to `Iso3166Alpha2` and `Iso3166Alpha3`.
  - `FromStr` implementations for each enum to parse from strings.
  - `Display` implementations for easy printing.
  - Utility methods like `Country::alpha2()` and `Country::alpha3()` for quick code lookups.
  
- **Serde Integration:**  
  All enums derive `Serialize` and `Deserialize`, making it easy to persist and exchange country data in JSON or other formats.

- **Extensible & Maintainable:**  
  Adding or modifying a country's mapping is as simple as editing a single macro invocation. No need to update multiple code sections.

## Example Usage

```rust
use country::{Country, Iso3166Alpha2, Iso3166Alpha3};
use std::str::FromStr;

fn main() {
    let c = Country::USA;
    println!("Country: {}", c);           // "USA"
    println!("Alpha-2: {:?}", c.alpha2()); // Some like US
    println!("Alpha-3: {:?}", c.alpha3()); // Some like USA

    let from_alpha2 = Iso3166Alpha2::from_str("GB").unwrap();
    println!("From Alpha-2: {:?}", from_alpha2); // GBR for Great Britain

    let from_country_str = Country::from_str("France").unwrap();
    println!("From Country str: {}", from_country_str); // "France"
}
```

## Use Cases

- **Localization & Internationalization:**  
  Quickly convert between human-friendly country names and standardized codes for UIs, APIs, and data storage.
  
- **Data Validation & Parsing:**  
  Validate country inputs from users, forms, or external services by parsing into `Country`, ensuring correctness and consistency.
  
- **Geopolitical & Mapping Apps:**  
  Serve as a robust backbone for applications that rely heavily on country data, enabling swift lookups, code conversions, and serialization.

## Adding a New Country or Modifying Codes

Update the macro invocation in the source code. All conversions and parsing logic will be regenerated automatically during compilation, ensuring no inconsistencies.

## License

This project is licensed under the MIT license. See the [LICENSE](LICENSE) file for details.
