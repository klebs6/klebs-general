
This crate provides the Abbreviation trait, common in the north-america, europe, south-america, etc crates

```rust
pub trait Abbreviation {
    fn abbreviation(&self) -> &'static str;
}
```
