# language-distribution

**language-distribution** is a Rust crate that integrates with the `country` and `language_enum` crates to provide language distribution data for each country. Given a `Country` enum variant (from the `country` crate), the crate returns a `HashMap<Language, f64>` indicating the relative prevalence of each language spoken within that country. Additionally, it provides a method to obtain a random language selection weighted by these distributions.

## Features

- **Country-to-Language Mappings:**  
  For each `Country` enum variant, a corresponding set of `(Language, f64)` pairs is provided, representing the proportion of speakers in that country.

- **Integration with `language_enum`:**  
  Uses the `Language` enum (from `language_enum`) as keys in the resulting `HashMap`, ensuring consistency with other language-related functionality.

- **Probability-Based Random Language Selection:**  
  Implements a `RandomLanguage` trait for `Country`, allowing selection of a random `Language` according to the weighted distribution obtained from `language_distribution()`.

- **Flexible Data Structures:**  
  Returns a `HashMap<Language, f64>` for easy integration with downstream code for analytics, UI display, or further processing.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
language-distribution = "0.1"
language-enum = "0.1"
country = "0.1"
rand = "0.8"
```

Then in your code:

```rust
use country::Country;
use language_distribution::{LanguageDistribution, RandomLanguage};

fn main() {
    let c = Country::Spain;
    let distribution = c.language_distribution();

    // distribution now contains a HashMap<Language, f64> with language prevalence
    for (lang, prevalence) in &distribution {
        println!("{:?}: {}", lang, prevalence);
    }

    // Get a random language based on the distribution
    if let Some(random_lang) = c.random_language() {
        println!("Random language for Spain: {:?}", random_lang);
    } else {
        println!("No languages found for this country.");
    }
}
```

## Data Sources and Accuracy

The language distributions are approximations, not meant to be authoritative. Adjust values as needed for more accuracy or updated data.

## Testing

An exhaustive test suite is included. Run tests using:

```bash
cargo test
```

The tests verify:

- The correctness of distributions for a sample of countries.
- The presence of expected languages for given countries.
- The random language selection method to ensure it returns valid languages from the distribution.

## License

This project is licensed under the MIT license. See [LICENSE](./LICENSE) for details.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request on GitHub.
