# str-shorthand

`str-shorthand` is a Rust crate that provides utility functions for string manipulation. The initial version (`0.1.0`) includes a function to bisect a string into two halves, handling multi-byte UTF-8 characters correctly.

## Features

- **Bisect Function**: Automatically splits a string into two halves, ensuring UTF-8 safety and correctness.

## Installation

Add `str-shorthand` to your `Cargo.toml`:

```toml
[dependencies]
str-shorthand = "0.1.0"
```

Then, include it in your project:

```rust
use str_shorthand::bisect;
```

## Usage

The bisect function splits a string into two halves, ensuring correct handling of multi-byte UTF-8 characters.

## Example

```rust
use str_shorthand::bisect;

fn main() {
    let text = "a😊bc😊";
    let (first_half, second_half) = bisect(text);
    println!("First half: {}", first_half); // Output: a😊b
    println!("Second half: {}", second_half); // Output: c😊
}
```
## Functions

`bisect`

Splits a string into two halves.

Signature:

```rust
pub fn bisect(text: &str) -> (&str, &str)
```

### Parameters:

`text`: A string slice to be split.

### Returns:

A tuple containing the two halves of the input string.

## Examples:

```rust
use str_shorthand::bisect;

let text = "abcdef";
let (first_half, second_half) = bisect(text);
assert_eq!(first_half, "abc");
assert_eq!(second_half, "def");

let text = "a😊b😊c";
let (first_half, second_half) = bisect(text);
assert_eq!(first_half, "a😊b");
assert_eq!(second_half, "😊c");
```

## Contributing

Contributions are welcome! Please open an issue or submit a pull request on GitHub.

## License

This project is licensed under the MIT License. See the LICENSE file for details.
