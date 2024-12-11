# month-and-season

**month-and-season** is a Rust library that provides typed enumerations for the 12 months of the year and the 4 meteorological seasons, along with utilities for converting between them, iterating through them, and performing basic navigation.

**Features:**

- `Month` enumeration:  
  - Convert between numeric representations (1-12) and `Month`.
  - Convert to/from string representations (e.g. `"January"` <-> `Month::January`).
  - Navigate through months with `next()` and `previous()`.
  - Iterate through all months.

- `Season` enumeration:  
  - Determine a `Season` from a given `Month` using `Season::from_month()`.
  - Convert to/from string representations (e.g. `"Autumn"` <-> `Season::Autumn`).
  - Retrieve all months in a season via `Season::months()`.
  - Iterate through all seasons.

**Example Usage:**
```rust
use month_and_season::{Month, Season};
use std::str::FromStr;

fn main() {
    let m = Month::March;
    println!("Month: {}", m);

    let s = Season::from_month(m);
    println!("Season for March: {}", s);

    let parsed_month = Month::from_str("October").unwrap();
    println!("Parsed month: {}", parsed_month);

    let parsed_season = Season::from_str("Winter").unwrap();
    println!("Parsed season: {}", parsed_season);

    println!("All months in Summer: {:?}", Season::Summer.months());
    println!("Next month after December: {}", Month::December.next());
}
```
