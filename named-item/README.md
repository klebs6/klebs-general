# named-item

`named-item` provides a set of traits for handling named items in Rust. It includes functionality for name management, validation, aliases, history tracking, and more. The crate is designed to help developers manage named entities flexibly and consistently in applications requiring robust name-handling capabilities.

## Features

- **Name Management**: Manage names with traits like `Named`, `SetName`, `ResetName`, and more.
- **Alias Support**: Track alternate names or aliases for an item.
- **Name History**: Maintain a history of name changes for audit or tracking purposes.
- **Name Validation**: Validate names using regular expressions or custom rules.
- **Serialization Support**: Optional support for `serde` to serialize and deserialize names.
- **Custom Names**: Use the `Other(String)` variant for names not included in the predefined list.

## Installation

To use the `named-item` crate, add it to your `Cargo.toml`:

```toml
[dependencies]
named-item = "0.1"
```

If you want to enable serde support for serialization and deserialization, include the feature:

```toml
[dependencies]
named-item = { version = "0.1", features = ["serde"] }
```

## Usage

### Basic Usage
You can define and manage names using the provided traits. Here is an example of how to create an item with a name, set a new name, and reset it to the default:

```rust
use named_item::{Named, SetName, ResetName, DefaultName};

struct Item {
    name: String,
}

impl Named for Item {
    fn name(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.name)
    }
}

impl SetName for Item {
    fn set_name(&mut self, name: &str) -> Result<(), ()> {
        self.name = name.to_string();
        Ok(())
    }
}

impl DefaultName for Item {
    fn default_name() -> Cow<'static, str> {
        Cow::Borrowed("Unnamed Item")
    }
}

impl ResetName for Item {}

fn main() {
    let mut item = Item { name: "Initial".to_string() };
    println!("Item name: {}", item.name());

    // Set a new name
    item.set_name("New Name").unwrap();
    println!("Updated name: {}", item.name());

    // Reset to default name
    item.reset_name().unwrap();
    println!("Reset name: {}", item.name());
}
```

### Aliases and Name History
The crate also provides functionality for tracking aliases and maintaining name histories:

```rust
use named_item::{Named, SetNameWithHistory, NameHistory};

struct TrackableItem {
    name: String,
    name_history: Vec<String>,
}

impl Named for TrackableItem {
    fn name(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.name)
    }
}

impl SetNameWithHistory for TrackableItem {
    fn set_name_with_history(&mut self, name: &str) -> Result<(), ()> {
        self.add_name_to_history(name);
        self.name = name.to_string();
        Ok(())
    }
}

impl NameHistory for TrackableItem {
    fn add_name_to_history(&mut self, name: &str) {
        self.name_history.push(name.to_string());
    }

    fn name_history(&self) -> Vec<Cow<'_, str>> {
        self.name_history.iter().map(|s| Cow::Borrowed(s.as_str())).collect()
    }
}

fn main() {
    let mut item = TrackableItem { name: "Initial".to_string(), name_history: vec![] };

    // Change name and track history
    item.set_name_with_history("New Name").unwrap();
    item.set_name_with_history("Another Name").unwrap();

    // Print name history
    println!("Name history: {:?}", item.name_history());
}
```

### Name Validation
You can validate names using custom logic or regular expressions with the ValidateName trait.

```rust
use named_item::{ValidateName, NameValidator};

fn main() {
    let validator = NameValidator::new(r"^[a-zA-Z]+$").unwrap();

    assert!(validator.validate_name("ValidName").is_ok());
    assert!(validator.validate_name("Invalid Name!").is_err());
}
```

### Traits

#### Named
Returns the current name of an item.

```rust
pub trait Named {
    fn name(&self) -> Cow<'_, str>;
}
```

#### SetName
Sets a new name for an item.

```rust
pub trait SetName {
    fn set_name(&mut self, name: &str) -> Result<(), ()>;
}
```

#### ResetName
Resets the name of an item to a default value.

```rust
pub trait ResetName: SetName + DefaultName {
    fn reset_name(&mut self) -> Result<(), ()> {
        self.set_name(&Self::default_name())
    }
}
```

#### DefaultName
Returns the default name of an item.

```rust
pub trait DefaultName {
    fn default_name() -> Cow<'static, str>;
}
```

#### NameHistory
Tracks the history of names an item has had.

```rust
pub trait NameHistory {
    fn add_name_to_history(&mut self, name: &str);
    fn name_history(&self) -> Vec<Cow<'_, str>>;
}
```

#### SetNameWithHistory
Extends SetName by adding name changes to the history.

```rust
pub trait SetNameWithHistory: SetName + NameHistory {
    fn set_name_with_history(&mut self, name: &str) -> Result<(), ()>;
}
```

#### ValidateName
Validates a name using custom logic or regular expressions.

```rust
pub trait ValidateName {
    fn validate_name(name: &str) -> Result<(), ()>;
}
```

### License

This crate is licensed under either of:

Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)
at your option.

### Contribution

Contributions are welcome! Feel free to submit a pull request or open an issue on GitHub.
