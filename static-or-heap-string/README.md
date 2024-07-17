# static-or-heap-string

`static-or-heap-string` is a Rust crate that provides an enum type, `StaticOrHeapString`, to handle strings that can either be static string slices (`&'static str`) or heap-allocated `String`s. This allows for flexible and efficient string handling in situations where either static or dynamic string data might be encountered.

## Features

- Supports serialization and deserialization with Serde.
- Implements comparison traits (`PartialEq`, `PartialOrd`, `Ord`).
- Implements hashing, ignoring the variant type.
- Provides a unified API for accessing the string data.

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
static-or-heap-string = "0.1.0"
serde = { version = "1.0", features = ["derive"] }
```

Import and use the StaticOrHeapString enum in your code:

```rust
use static_or_heap_string::StaticOrHeapString;
use serde_json;

fn main() {
    // Example usage
    let static_str = StaticOrHeapString::Static("hello");
    let heap_str = StaticOrHeapString::Heap(String::from("world"));

    println!("{:?}", static_str);
    println!("{:?}", heap_str);

    // Serialize and deserialize
    let serialized = serde_json::to_string(&static_str).unwrap();
    let deserialized: StaticOrHeapString = serde_json::from_str(&serialized).unwrap();
    assert_eq!(static_str, deserialized);
}
```

## Enum Variants

```rust
pub enum StaticOrHeapString {
    Static(&'static str),
    Heap(String),
}
```

`Static(&'static str)`: Represents a static string slice.
`Heap(String)`: Represents a heap-allocated string.

## Methods
`as_str`
Returns the string slice representation of the enum variant.

```rust
impl StaticOrHeapString {
    pub fn as_str(&self) -> &str {
        match self {
            StaticOrHeapString::Static(s) => s,
            StaticOrHeapString::Heap(s) => s.as_str(),
        }
    }
}
```

## Trait Implementations
`PartialEq`, `Eq`: Compare the string content, ignoring the variant.
`PartialOrd`, `Ord`: Compare the string content lexicographically.
`Hash`: Hash the string content, ignoring the variant.
`Debug`: Format the string content for debugging.
`Clone`: Clone the enum, preserving the variant.
`Serialize`, `Deserialize`: Serialize and deserialize the string content using Serde.

## License
This project is licensed under the MIT License. See the LICENSE file for details.

## Contributing
Contributions are welcome! Please open an issue or submit a pull request on GitHub.
