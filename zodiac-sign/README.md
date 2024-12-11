# zodiac-sign

**zodiac-sign** is a Rust library providing a typed enumeration of the 12 Western astrological zodiac signs, along with utilities for converting from month/day dates, string representations, and iterating through the signs.

**Features:**

- Enumerates all 12 zodiac signs: Aries through Pisces.
- Convert `ZodiacSign` to/from `&str` for easy serialization or human-readable output.
- Determine a sign from a given month/day combination (`ZodiacSign::from_month_day`).
- Access the canonical date range for each sign (`ZodiacSign::date_range`).
- Iterate through all signs.
- `Serialize`/`Deserialize` support via Serde.

**Example:**

```rust
use zodiac_sign::ZodiacSign;
use std::str::FromStr;

fn main() {
    let sign = ZodiacSign::from_month_day(3, 21).unwrap(); // March 21
    println!("March 21 sign: {}", sign); // Aries

    let parsed = ZodiacSign::from_str("Leo").unwrap();
    println!("Parsed: {}", parsed);

    let (start, end) = sign.date_range();
    println!("Aries range: {:?} - {:?}", start, end);
}

