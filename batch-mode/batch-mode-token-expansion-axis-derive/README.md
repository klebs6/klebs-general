# batch-mode-token-expansion-axis-derive

This crate provides a procedural macro `#[derive(TokenExpansionAxis)]` for enumerations, enabling automatic implementation of axis-related traits from the [`batch-mode-token-expansion-axis`](https://github.com/klebs6/klebs-general) crate. By specifying attributes like `#[system_message_goal("...")]` and `#[axis("axis_name => axis_description")]`, you can generate:

1. **AxisName** and **AxisDescription** implementations for each variant.
2. A data-carrying struct that stores expanded data (e.g., `Expanded...`).
3. An aggregator struct that implements `TokenExpander` and `SystemMessageGoal`, tying the process together.

## Features

- **system_message_goal attribute**: Annotate the enum with a default or custom system message goal (e.g., `#[system_message_goal("Convert tokens to JSON")]`).
- **axis attribute**: For each variant, specify an axis name and description (e.g., `#[axis("my_axis => Provides an axis for expansion")]`).
- **Generated Aggregator**: Automatically implements `Default`, `Named`, `TokenExpander`, and `SystemMessageGoal`, facilitating batch expansions.
- **Data-carrying Struct**: A structured result type (e.g., `Expanded...`) that includes fields derived from your enumeration variants.

### Example Usage

```rust
use batch_mode_token_expansion_axis_derive::TokenExpansionAxis;
use std::borrow::Cow;

#[derive(TokenExpansionAxis, Debug)]
#[system_message_goal("Transform tokens along multiple axes")]
enum ExampleExpanderAxis {
    #[axis("lang => Expands the language dimension")]
    Language,
    #[axis("style => Expands the style dimension")]
    Style,
}
```

When this derives, it generates:
1. Implementations for `AxisName`, `AxisDescription`, and `TokenExpansionAxis` on `ExampleExpanderAxis`.
2. A data-carrying struct named `ExpandedExampleExpander` (if you strip off the `ExpanderAxis` suffix, it becomes `ExpandedExample`—the macro handles naming automatically).
3. An aggregator struct named `ExampleExpander`, which implements traits like `TokenExpander` and `SystemMessageGoal`.

## Getting Started

1. **Add to `Cargo.toml`**:
   ```toml
   [dependencies]
   batch-mode-token-expansion-axis = "0.1"
   batch-mode-token-expansion-axis-derive = "0.1"
   ```
2. **Import and Use**:
   ```rust
   use batch_mode_token_expansion_axis_derive::TokenExpansionAxis;
   use batch_mode_token_expansion_axis::{TokenExpansionAxis, AxisName, AxisDescription};

   #[derive(TokenExpansionAxis)]
   enum MyAxis {
       #[axis("color => Expand color variations")]
       Color,
       #[axis("size => Expand size variations")]
       Size,
   }
   ```

3. **Leverage the Generated Code**:
   - Instantiating the aggregator (e.g., `MyExpander`) allows accessing all enum variants as `Arc<dyn TokenExpansionAxis>`.
   - The expanded struct (e.g., `ExpandedMy`) supports serialization, deserialization, and other derived functionality.

This library is designed to integrate seamlessly with `batch-mode-token-expansion-axis`, providing a clean, DRY approach to multi-dimensional token expansions.
