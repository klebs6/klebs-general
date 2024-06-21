# export-magic

`export-magic` is a Rust crate designed to simplify module management and re-exportation using macros. 

It provides a set of powerful macros that allow developers to effortlessly manage module imports and exports, reducing boilerplate and improving code organization.

## Features

- `x!`: Automatically define and re-export everything from a specified module.
- `xp!`: Define a module and use its items locally without re-exporting them.
- `ix!`: Import all items from the crate root and a specific `imports` module with a single macro call.

## Example Usage

### Using xp! Macro

If you want to use all items from a module bar within another module without re-exporting them:

```rust
xp!(bar);
```

Typically, this is useful inside of the `src/lib.rs` file within a proc_macro crate.

For example, in the `error-tree` proc_macro crate, we have the following `lib.rs` file:

```rust
extern crate proc_macro;

#[macro_use] mod imports; use imports::*;

xp!{conversion_chain}
xp!{error_enum}
xp!{error_field}
xp!{error_tree_parse}
xp!{error_tree_visitor}
xp!{error_tree}
xp!{error_variant}
xp!{errors}
xp!{from_impl_generation_config}
xp!{from_impl_generation_config_emitter}
xp!{types}
xp!{validate}

#[proc_macro]
pub fn error_tree(input: TokenStream) -> TokenStream {

    let error_tree = parse_macro_input!(input as ErrorTree);

    error_tree.into_token_stream().into()
}
```

Here, we use the `xp!` macro calls to make the models found in the listed source files available within our `error_tree` function call. 

This works without needing to expose these underlying models to the `error_tree` crate consumer.

### Using ix! Macro

To import everything from your crate's root and an imports module into the current scope:

```rust
ix!();
```

We typically use this macro call at the top of each source file to make the publically defined neighboring items available within our file without boilerplate. 

It also brings in third party items from `src/imports.rs` into the calling source file.

In the `xp!` example above, we use `crate::ix!()` at the top of each file (conversion_chain.rs, error_enum.rs, error_field.rs, etc).

Thus, we can write the whole system without much boilerplate for use statements.

Typically, `src/imports.rs` will look like this:

```rust
pub(crate) use std::sync::Arc;
pub(crate) use std::fs::*;
```

Whatever we specify inside of `src/imports.rs` will become available to any `src/sourcefile.rs` which calls `crate::ix!()` at the top.

### Using `x!` Macro
Suppose you have a module `foo` with various public structs and functions, and you want to make them available in your library's root. Simply use:

```rust
x!(foo);
```

This lets us cleanly and easily organize our crates to make their contents available to their clients.
Typically, our `src/lib.rs` files may look just like this, no more, no less:

```rust
#[macro_use] mod imports; use imports::*;

x!{errors}
x!{traits}
x!{sourcefile1}
x!{sourcefile2}
x!{sourcefile3}
```

Within each of the files `src/errors.rs`, `src/traits.rs`, `src/sourcefile1.rs`, `src/sourcefile2.rs`, etc we will have `crate::ix!()` specified at the top (see the section above).

Any third party dependency is exported with `pub(crate) use some_third_party_dep::*` within `src/imports.rs` and so all of our source files become super clean.

### Installation

Add this to your Cargo.toml:

```toml
[dependencies]
export-magic = "0.1.0"
```

### License

This crate is licensed under the MIT License.
