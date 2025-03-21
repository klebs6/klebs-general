# batch-mode-token-expansion-traits

This crate provides trait definitions for describing and implementing token expansion axes. It is particularly useful when orchestrating batch-based expansions for language models or similar systems. By defining an axis, you specify a distinct dimension or perspective along which a token can be expanded.

## Features

- **AxisName**: Associates each axis variant with a concise, programmatic name string.
- **AxisDescription**: Provides a descriptive prompt or explanation, typically used to guide language model expansions.
- **TokenExpansionAxis**: Combines both naming and descriptive functionality, establishing a standard interface for axis-related traits.

### Example Usage

Implement these traits on an enum or struct to define multiple axes for token expansion:

```rust
use std::borrow::Cow;
use std::fmt::Debug;

#[derive(Debug)]
enum MyAxis {
    VariantOne,
    VariantTwo,
}

impl crate::AxisName for MyAxis {
    fn axis_name(&self) -> Cow<'_, str> {
        match self {
            MyAxis::VariantOne => "variant_one".into(),
            MyAxis::VariantTwo => "variant_two".into(),
        }
    }
}

impl crate::AxisDescription for MyAxis {
    fn axis_description(&self) -> Cow<'_, str> {
        match self {
            MyAxis::VariantOne => "Instructions or description for VariantOne".into(),
            MyAxis::VariantTwo => "Instructions or description for VariantTwo".into(),
        }
    }
}

impl crate::TokenExpansionAxis for MyAxis {}
```

You can now treat `MyAxis` as a token expansion axis, iterating over each variant to build prompts or instructions for your batch-based expansions.

## Getting Started

1. Add this crate to your `Cargo.toml`.
2. Use the provided traits to define your own axes.
3. Integrate them into your batch expansion flow.
