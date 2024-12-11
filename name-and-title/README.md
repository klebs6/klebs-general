# name-and-title

**name-and-title** is a Rust library providing a strongly typed representation of a person's name, including an optional title, first name, middle name (or initial), and last name. It leverages builder patterns, `serde` for serialization/deserialization, and convenience macros to simplify name construction and usage in your applications.

**Key Features:**

- **Flexible Name Representation:**  
  Supports `Option`al title, middle name/initials, and fully typed `FirstName`, `MiddleName`, `LastName` structures.

- **Convenience Macros:**  
  Quickly convert string literals or characters into `FirstName`, `LastName`, and `Middle` variants. For example:
  ```rust
  let name = PersonNameBuilder::default()
      .first("Alice")
      .middle("Marie")   // or .middle('M')
      .last("Smith")
      .build()
      .unwrap();
  ```

- **Optional Title with Defaults:**  
  The `title` field is fully optional. If omitted, it defaults to `None`, allowing for simple "first last" formats.

- **Integration with `serde`:**  
  Serialize and deserialize names to/from JSON and other formats. This makes it easy to store and retrieve user information in databases, files, or transmit over networks.

- **`FromStr` and `Display` for Titles:**  
  Person titles such as "Mr.", "Mrs.", "Dr." can be parsed from strings and displayed back easily, including custom or unknown titles.

**Example:**

```rust
use name_and_title::{PersonNameBuilder, PersonTitle};

fn main() {
    let name = PersonNameBuilder::default()
        .title(PersonTitle::Dr)
        .first("John")
        .middle("Q")
        .last("Public")
        .build()
        .expect("Failed to build PersonName");

    println!("Full name: {}", name.full_name());
    // Outputs: "Dr. John Q Public"

    // Without title and middle:
    let simple_name = PersonNameBuilder::default()
        .first("Jane")
        .last("Doe")
        .build()
        .expect("Failed to build PersonName");

    println!("Simple name: {}", simple_name.full_name());
    // Outputs: "Jane Doe"
}
```

**Use Cases:**

- **User Interfaces:** Present names formally or informally in UI elements.
- **E-Commerce/CRM Systems:** Store and retrieve customer names with optional titles.
- **Localization & Formatting:** Integrate with translation or formatting systems to respect cultural naming conventions.
- **Data Serialization:** Easily move between Rust structs and stored JSON data for user profiles.

**License:**

This project is licensed under the MIT license. See [LICENSE](LICENSE) for details.
