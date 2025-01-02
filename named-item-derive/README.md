# named-item-derive

A single **procedural macro** crate that integrates with [`named-item`](https://github.com/YourUsername/named-item) to provide robust name-management for Rust structs.

## Features

- **`#[derive(NamedItem)]`**: The single macro that implements:
  - `Named`, `SetName`, `DefaultName`, `ResetName`
  - Conditionally implements `NameHistory` if `#[named_item(history="true")]` is specified.
  - Conditionally implements `NamedAlias` (including `clear_aliases`) if `#[named_item(aliases="true")]` is specified.
- Supports **custom default names** via `#[named_item(default_name="...")]`.
- Supports **default aliases** via `#[named_item(default_aliases="foo,bar")]`.
- Enforces a `name: String` field. If `history="true"`, it also requires `name_history: Vec<String>`; if `aliases="true"`, requires `aliases: Vec<String>`.

## Usage

First, add this crate and its companion `named-item` as dependencies in your `Cargo.toml`:

```toml
[dependencies]
named-item = "x.y.z"
named-item-derive = { path = "../named-item-derive" }  # or your version
```

Then, in your code:

```rust
use named_item_derive::NamedItem;
use named_item::{Named, SetName, ResetName, NameAlias, NameHistory, NameError};

#[derive(NamedItem)]
#[named_item(
    default_name="TomeOfSecrets",
    aliases="true",
    default_aliases="alpha,beta",
    history="true"
)]
pub struct MagicalTome {
    pub name: String,
    pub name_history: Vec<String>,
    pub aliases: Vec<String>,
}

fn main() -> Result<(), NameError> {
    let mut tome = MagicalTome {
        name: "Prototype".to_string(),
        name_history: vec![],
        aliases: vec![],
    };

    // Standard name handling
    tome.set_name("Revised Tome")?;
    assert_eq!(tome.name(), "Revised Tome");

    // NameHistory tracking
    assert_eq!(tome.name_history(), vec!["Revised Tome"]);

    // NamedAlias
    tome.add_alias("Secret Volume");
    assert_eq!(tome.aliases(), vec!["Secret Volume"]);
    tome.clear_aliases();
    assert!(tome.aliases().is_empty());

    // Reset to default => "TomeOfSecrets"
    tome.reset_name()?;
    assert_eq!(tome.name(), "TomeOfSecrets");

    // Default aliases => ["alpha", "beta"]
    let defaults = MagicalTome::default_aliases();
    assert_eq!(defaults, vec!["alpha", "beta"]);

    Ok(())
}
```

When `history="true"`, **each** call to `set_name` automatically appends the new name to `name_history`, while `aliases="true"` implements the `NamedAlias` trait with `add_alias`, `aliases`, and `clear_aliases`.

## Tests

We include:
- **Integration tests** under `tests/integration.rs`.
- Optionally, **UI tests** (compile-fail) with [trybuild](https://github.com/dtolnay/trybuild) in `tests/ui/`.

Run:
```bash
cargo test
```
to ensure all tests pass.

## License

This project is licensed under the MIT license. See [LICENSE](LICENSE) for details.

## Contributing

Pull requests are welcome! Please open an issue for major changes to discuss them first.
