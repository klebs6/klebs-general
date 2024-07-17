
# find-matching-bracket

`find-matching-bracket` is a Rust crate that helps you find the matching closing bracket for a given opening bracket in a string. It supports curly braces, square brackets, and parentheses. This crate is useful for parsing code, validating expressions, and more.

## Features

- Supports curly braces `{}`, square brackets `[]`, and parentheses `()`.
- Handles nested brackets.
- Handles large inputs efficiently.
- Supports Unicode characters.

## Usage

Add this crate to your `Cargo.toml`:

```toml
[dependencies]
find-matching-bracket = "0.1.0"
```

## Example

```rust
use find_matching_bracket::{find_matching_curly, find_matching_square, find_matching_paren};

fn main() {
    let text = "{content}";
    let position = find_matching_curly(text, 0);
    println!("Matching position: {:?}", position); // Output: Matching position: Some(8)
}
```

## Functions

### `find_matching_bracket`
Finds the matching closing bracket for the given opening bracket in a string.

#### Parameters

- `text`: The input string.
- `start`: The starting position of the opening bracket.
- `bracket_type`: The type of bracket (Curly, Square, or Paren).

#### Returns
- `Option<usize>`: The position of the matching closing bracket, or None if no matching bracket is found.

### `find_matching_curly_bracket`
Finds the matching closing curly bracket {} for the given opening bracket in a string.

#### Parameters
- `text`: The input string.
- `start`: The starting position of the opening curly bracket.

#### Returns
- `Option<usize>`: The position of the matching closing bracket, or None if no matching bracket is found.

### `find_matching_square_bracket`
Finds the matching closing square bracket [] for the given opening bracket in a string.

#### Parameters
- `text`: The input string.
- `start`: The starting position of the opening square bracket.

#### Returns
- `Option<usize>`: The position of the matching closing bracket, or None if no matching bracket is found.

### `find_matching_paren`
Finds the matching closing parenthesis () for the given opening bracket in a string.

#### Parameters
- `text`: The input string.
- `start`: The starting position of the opening parenthesis.

#### Returns
- `Option<usize>`: The position of the matching closing bracket, or None if no matching bracket is found.

## Example
```rust
let text = "{[()][]}";
let position = find_matching_curly(text, 0);
assert_eq!(position, Some(7));
```

## License
This project is licensed under the MIT License.
