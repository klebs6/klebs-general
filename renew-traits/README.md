# renew-traits

`renew-traits` provides a set of utility traits to manage, initialize, and extend collections or other data structures in a consistent and reusable way. The crate is designed to help you define flexible and extensible APIs for managing internal collections and structures.

## Traits

### 1. `ExtendWith`
Extends a collection with a single item, handling potential errors.

```rust
pub trait ExtendWith<Item> {
    type Error;

    fn extend_with(&mut self, item: &Item) -> Result<(), Self::Error>;
}

```

### 2. `FillToLenWithItems`
Fills a collection with a list of items until it reaches a specified length.

```rust
pub trait FillToLenWithItems {
    type Item;

    fn fill_to_len(&mut self, len: usize, items: Vec<Self::Item>);
}
```

### 3. `ReinitWithLen`
Reinitializes a collection or structure to a specified length.

```rust
pub trait ReinitWithLen {
    fn reinit(&mut self, len: usize);
}
```

### 4. `FillWith`
Fills a collection with a single value across all its elements.

```rust
pub trait FillWith {
    type Item;

    fn fill(&mut self, val: Self::Item);
}
```

### 5. `InitInternals`
Initializes internal data structures, returning a result to handle potential errors.

```rust
pub trait InitInternals {
    type Error;

    fn init_internals(&mut self) -> Result<(), Self::Error>;
}
```

### 6. `InitWithSize`
Initializes the internal structure with a given size.

```rust
pub trait InitWithSize {
    fn init_size(&mut self, size: usize);
}
```

### 7. `Clear`
Clears all elements from the collection or structure.

```rust
pub trait Clear {
    fn clear(&mut self);
}
```

### 8. `CreateNamedEmpty`
Creates an empty instance with an associated name.

```rust
pub trait CreateNamedEmpty {
    fn empty(name: &str) -> Self;
}
```

### 9. `CreateEmpty`
Creates an empty instance of a collection or structure.

```rust
pub trait CreateEmpty {
    fn empty() -> Self;
}
```

### 10. `ResetWith`
Resets the collection or structure using a provided input.

```rust
pub trait ResetWith<Input> {
    fn reset_with(&mut self, g: &Input);
}
```

### Installation

Add renew-traits to your Cargo.toml:

```toml
[dependencies]
renew-traits = "0.1"
```

### Usage

Here's an example of how to implement some of the traits:

```rust
struct MyCollection {
    data: Vec<i32>,
    name: String,
}

impl CreateNamedEmpty for MyCollection {
    fn empty(name: &str) -> Self {
        Self { data: Vec::new(), name: name.to_string() }
    }
}

impl ExtendWith<i32> for MyCollection {
    type Error = ();

    fn extend_with(&mut self, item: &i32) -> Result<(), Self::Error> {
        self.data.push(*item);
        Ok(())
    }
}
```

### License

This crate is licensed under either of:

Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)
at your option.

### Contributing

Contributions are welcome! Feel free to submit a pull request or open an issue on GitHub.
