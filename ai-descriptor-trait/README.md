# ai-descriptor-trait

`ai-descriptor-trait` is a Rust crate that provides a simple, structured way to describe items with features in a human-readable format using AI-style descriptors. It defines reusable traits to represent items and their features, then generates a formatted description for these items.

## Features

- **Modular Traits**: 
  - `ItemFeature`: Represents an individual feature with a text description.
  - `ItemWithFeatures`: Represents an item containing a header and a collection of features.
  - `AIDescriptor`: Automatically formats an item's header and its features into a human-readable string.

- **Convenience**: Automatically composes structured descriptions for any type implementing `ItemWithFeatures`.

- **Flexibility**: Uses `Cow<'_, str>` for efficient memory management, allowing seamless use of owned and borrowed strings.

## Example Usage

Below is an example of how to implement and use the provided traits:

```rust
use std::borrow::Cow;
use ai_descriptor_trait::{ItemWithFeatures, AIDescriptor};

struct MyItem {
    header: String,
    features: Vec<Cow<'static, str>>,
}

impl ItemWithFeatures for MyItem {
    fn header(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.header)
    }

    fn features(&self) -> &[Cow<'_, str>] {
        &self.features
    }
}

fn main() {
    let item = MyItem {
        header: "My Cool Item".to_string(),
        features: vec![
            Cow::Borrowed("Lightweight"),
            Cow::Borrowed("Durable"),
            Cow::Borrowed("Eco-friendly"),
        ],
    };

    println!("{}", item.ai());
}
```

**Output**:
```
My Cool Item
It has the following features:
- Lightweight
- Durable
- Eco-friendly
```

## Integration with Your Project

Add the following to your `Cargo.toml`:

```toml
[dependencies]
ai-descriptor-trait = "0.1.0"
```

## Running Tests

Unit tests are provided to ensure the functionality of the `AIDescriptor` implementation. Run the tests using:

```bash
cargo test
```

## Key Concepts

### Traits Overview

- **`ItemFeature`**:
  Defines a single feature for an item.

  ```rust
  pub trait ItemFeature {
      fn text(&self) -> Cow<'_, str>;
  }
  ```

- **`ItemWithFeatures`**:
  Represents an item with a header and a collection of features.

  ```rust
  pub trait ItemWithFeatures {
      fn header(&self) -> Cow<'_, str>;
      fn features(&self) -> &[Cow<'_, str>];
  }
  ```

- **`AIDescriptor`**:
  Generates a formatted description of any item implementing `ItemWithFeatures`.

  ```rust
  pub trait AIDescriptor {
      fn ai(&self) -> Cow<'_, str>;
  }
  ```

### Test Example

A test is provided in the module to ensure the `AIDescriptor` works as expected:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    struct TestItem {
        header: String,
        features: Vec<Cow<'static, str>>,
    }

    impl ItemWithFeatures for TestItem {
        fn header(&self) -> Cow<'_, str> {
            Cow::Borrowed(&self.header)
        }

        fn features(&self) -> &[Cow<'_, str>] {
            &self.features
        }
    }

    #[test]
    fn test_ai_descriptor() {
        let item = TestItem {
            header: "An Item.".to_string(),
            features: vec![
                Cow::Borrowed("Feature 1"),
                Cow::Borrowed("Feature 2"),
                Cow::Borrowed("Feature 3"),
            ],
        };

        let expected_output = "\
An Item.
It has the following features:
- Feature 1
- Feature 2
- Feature 3";

        assert_eq!(item.ai(), expected_output);
    }
}
```

## License

This crate is licensed under the [MIT License](LICENSE).

---

Contributions are welcome! Feel free to fork the repository and submit pull requests.
