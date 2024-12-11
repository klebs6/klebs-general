
# birthday-struct

**birthday-struct** is a Rust library designed for working with birthdays in the context of an online bookstore or any e-commerce system. It supports:

- Representing birthdays with an associated time zone.
- Calculating age based on the current date.
- Checking how many days until the next birthday or since the last one.
- Determining if today is the user's birthday.
- Identifying the zodiac sign corresponding to a given birthday.
- Providing localized birthday greetings in various languages.
- Efficient serialization and deserialization of birthday and timezone data via Serde.

## Features

- **Age Calculation:**  
  Compute a person's age from their birth date and today's date.
  
- **Next and Last Birthday:**  
  Find out how many days remain until the next birthday, or how many days have passed since the last one.

- **Is Today Their Birthday?:**  
  Quickly check if a birthday matches today's date.

- **Zodiac Sign Integration:**  
  Convert birthdays to their corresponding zodiac signs using the [`zodiac-sign`](https://crates.io/crates/zodiac-sign) crate.

- **Multilingual Greetings:**  
  Retrieve a birthday greeting in several languages via the [`language-enum`](https://crates.io/crates/language-enum) crate.

- **Serialization and Time Zones:**  
  Serialize and deserialize birthday information, including time zones provided by [`chrono-tz`](https://crates.io/crates/chrono-tz).

## Example

```rust
use storefront_birthday::{BirthdayBuilder, SerializableTimeZone, birthday_greeting};
use language_enum::Language;

fn main() {
    let birthday = BirthdayBuilder::default()
        .day(10)
        .month(8)
        .year(1990)
        .time_zone(SerializableTimeZone::utc())
        .build()
        .unwrap();

    println!("Is today their birthday? {}", birthday.is_today());
    println!("They are {} years old.", birthday.age().unwrap_or(0));
    println!("Days until next birthday: {}", birthday.days_until_next());
    println!("Days since last birthday: {}", birthday.days_since_last());

    let greeting = birthday_greeting(&Language::English).unwrap_or("Hello!");
    println!("{}", greeting);

    let zodiac: zodiac_sign::ZodiacSign = birthday.clone().into();
    println!("Their zodiac sign is: {}", zodiac);
}
```

## Installation
Add birthday-struct to your Cargo.toml:

```toml
[dependencies]
birthday-struct = "0.1.0"
```

## License
This project is licensed under the MIT license. See the LICENSE file for details.
