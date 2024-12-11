# postal-code

**postal-code** is a robust, production-grade Rust library for validating and manipulating international postal codes. It supports a variety of countries, including the USA, Canada, UK, France, Germany, and Italy, with the flexibility to add more.

This crate offers:

- Strongly typed `Country` integration from an external `country` crate.
- Compile-time verified regular expressions for each supported country's postal code format.
- Builder patterns (`derive_builder`) for safer, more ergonomic construction of `PostalCode` and `PostalCodeCollection` instances.
- A modular, extensible validator system using traits and macros to easily add new countries.
- Comprehensive error handling via a well-structured `PostalCodeConstructionError` type.
- A robust test suite that covers valid and invalid postal codes, edge cases, and collection handling.

**Key Features:**

- **Type-Safe:** Each postal code is associated with a `Country` enum variant, ensuring that once created, it is guaranteed to be valid for that country.
- **Extensible:** Add new countries by defining regex patterns and calling a macro to generate validators.
- **Error Handling:** Strong, typed errors (`PostalCodeConstructionError`) avoid stringly-typed error handling.
- **Testing and Reliability:** Thorough tests ensure that edge cases and invalid inputs are handled correctly.

**Usage Example:**

```rust
use postal_code::PostalCode;
use country::Country;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let us_code = PostalCode::new(Country::USA, "12345")?;
    println!("Valid US postal code: {}", us_code.code());

    let ca_code = PostalCode::new(Country::Canada, "K1A0B1")?;
    println!("Valid Canadian postal code: {}", ca_code.code());

    // Attempting an invalid UK code:
    let uk_code = PostalCode::new(Country::UK, "SW1A1AAZ");
    match uk_code {
        Ok(code) => println!("Valid UK code: {}", code.code()),
        Err(e) => eprintln!("Invalid postal code: {:?}", e),
    }

    Ok(())
}
```

**Testing:**

Run the test suite with:
```bash
cargo test
```

This will run an extensive battery of tests, including correctness checks for various countries, invalid inputs, and builder patterns.

**Contributing:**

Contributions are welcome! Please open issues or submit pull requests on the project’s GitHub repository.

**License:**

`postal-code` is licensed under the MIT license. See [LICENSE](LICENSE) for details.

