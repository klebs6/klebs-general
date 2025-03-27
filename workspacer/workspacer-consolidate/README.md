# workspacer-consolidate

`workspacer-consolidate` is a Rust library for aggregating and managing complex crate interfaces with precision and efficiency. It provides utilities for collecting and structuring various coding elements, such as functions, structs, enums, and other constructs, into a comprehensive and organized interface.

## Features

- **Consolidated Interfaces:** Merge and manage crate elements like functions, structs, enums, and traits into a single cohesive structure.
- **Customization Options:** Utilize `ConsolidationOptions` for custom consolidation strategies, deciding which elements to include or omit.
- **Attribute and Documentation Management:** Extract and unify documentation and attributes from code items for a consistent representation.
- **Test Item Handling:** Flexibly include or exclude test items based on configuration, aiding in streamlined interface presentation.
- **Trait and Module Management:** Seamlessly handle trait implementations and nested modules with robust management logic.

## Usage

This crate allows developers to consolidate their crate's interface elements in a highly configurable manner. To achieve this, the `ConsolidateCrateInterface` trait can be implemented and executed asynchronously:

```rust
#[async_trait]
trait ConsolidateCrateInterface {
    async fn consolidate_crate_interface(&self, options: &ConsolidationOptions) -> Result<ConsolidatedCrateInterface, CrateError>;
}
```

The consolidation process aggregates different elements based on detailed settings provided through `ConsolidationOptions`. You can manipulate the inclusion of documentation, private items, test items, and function bodies. Here's a brief configuration example:

```rust
let options = ConsolidationOptions::new()
    .with_docs()
    .with_private_items()
    .with_fn_bodies();
```

## Licensing

This project is dual-licensed under the MIT and Apache-2.0 licenses.

## Repository

Code and contribution guidelines are available on [GitHub](https://github.com/klebs6/klebs-general).

## Contributions

We welcome community contributions and encourage participation of all forms. Kindly refer to the contribution guide in our GitHub repository.