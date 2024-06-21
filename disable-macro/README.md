# disable

`disable` is a Rust procedural macro that allows you to conditionally disable blocks of code by completely removing them. This can be useful for debugging, feature gating, or simply turning off certain functionalities during development.

## Features

- Completely remove the annotated item from the compiled code with a single attribute.
- Easy integration into existing projects.

This is mostly useful (IMO) for easily disabling tests without block commenting them out.

## Usage

### Add to Your Project

Add `disable` as a dependency in your `Cargo.toml`:

```toml
[dependencies]
disable = "*"
```

## Example
Annotate the functions you want to disable with the #[disable] attribute

```rust
use disable::disable;

#[disable]
fn some_function() {
    println!("This code will not be compiled.");
}

// this will not compile, because `some_function` is disabled
fn main() {
    some_function(); 
}
```

## License
This project is licensed under the MIT License. See the LICENSE file for details.
