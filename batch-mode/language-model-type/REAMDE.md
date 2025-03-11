# language-model-type

This crate defines an enumeration of supported language model types (e.g. `gpt-4o`, `o1-preview`) and provides convenience traits for serialization, deserialization, and display. It can serve as a foundational component in systems needing a canonical reference for various model identifiers.

## Features

- **Enum Declaration**  
  - Enumerates distinct model variants such as `Gpt4o`, `Gpt4oMini`, `Gpt4Turbo`, etc.
  - Offers a typed approach for referencing models in a strongly typed system.

- **Serialization/Deserialization**  
  - Implements custom Serde logic for converting between enum variants and their string representations (e.g., `"gpt-4o"`).

- **Display Implementation**  
  - Provides a `std::fmt::Display` trait implementation, enabling easy string conversion for logs and user interfaces.

## Example Usage

```rust
use language_model_type::LanguageModelType;

fn main() {
    // Directly reference the enumeration:
    let model = LanguageModelType::Gpt4Turbo;

    // Display implementation
    println!("Model is: {}", model);

    // Convert to/from string (via Serde or manually):
    let as_string = serde_json::to_string(&model).unwrap();
    println!("Serialized: {}", as_string);

    let deserialized: LanguageModelType = serde_json::from_str(&as_string).unwrap();
    println!("Deserialized: {:?}", deserialized);
}
```
