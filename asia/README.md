# Asia Crate

The **asia** crate provides a rich enumeration of Asian countries and their subregions, with functionality similar to the `europe` crate. It includes:

- A comprehensive `AsiaRegion` enum representing both individual countries and subdivided countries.
- Conversion traits to convert between `AsiaRegion` and standard ISO country codes.
- Round-trip conversions from `AsiaRegion` to `Country` (from a shared `country` crate) and back.
- Serialization/Deserialization (Serde) support for `AsiaRegion` and its subregions, including nested subregions like `ChinaRegion`, `IndiaRegion`, `JapanRegion`, and `IndonesiaRegion`.
- Exhaustive test suites ensuring correctness and stability.

## Features

- **No unsafe code**: The codebase is entirely safe Rust.
- **No `unwrap` or `expect`**: All error handling is done gracefully with `Result` and custom error types.
- **No `thiserror`**: Error handling is implemented manually for full control and consistency.

## Examples

Convert a `AsiaRegion` to a `Country`:

```rust
use asia::{AsiaRegion, Country};
use std::convert::TryInto;

let region = AsiaRegion::Japan(asia::JapanRegion::Hokkaido);
let country: Country = region.try_into().expect("Should map to Japan");
assert_eq!(country.to_string(), "Japan");
```

Convert a `Country` back to `AsiaRegion`:

```rust
use asia::{AsiaRegion, Country};
use std::convert::TryInto;

let country = Country::India;
let region: AsiaRegion = country.try_into().expect("Should map to India");
assert!(matches!(region, AsiaRegion::India(_)));
```

Serialize and deserialize `AsiaRegion` with JSON:

```rust
use asia::AsiaRegion;
use asia::ChinaRegion;
use serde_json;

let region = AsiaRegion::China(ChinaRegion::Beijing);
let json = serde_json::to_string(&region).expect("serialize");
assert!(json.contains("\"country\":\"China\""));
assert!(json.contains("\"region\":\"Beijing\""));

let deser: AsiaRegion = serde_json::from_str(&json).expect("deserialize");
assert_eq!(deser, region);
```

## Error Handling

When attempting conversions that are not possible (e.g., mapping a combined region like GCC States to a single `Country`), a custom error type `AsiaRegionConversionError` is returned, allowing you to handle these cases gracefully.

## Contributing

Contributions, issues, and feature requests are welcome! Feel free to check the [issues page](https://github.com/yourusername/asia/issues).

1. Fork it!
2. Create your feature branch: `git checkout -b my-new-feature`
3. Commit your changes: `git commit -am 'Add some feature'`
4. Push to the branch: `git push origin my-new-feature`
5. Submit a pull request

## License

This project is licensed under the MIT license. See [LICENSE](LICENSE) for details.
