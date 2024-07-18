# count-invert

`count-invert` is a Rust crate providing utility functions for counting the occurrences of elements in a vector and inverting a `HashMap` based on those counts. This crate leverages the `itertools` crate to offer robust and efficient functionalities for common data manipulation tasks.

## Features

- Convert a vector into a `HashMap` with element counts.
- Invert a `HashMap` to map counts to vectors of elements.
- Combine both functionalities to create a `HashMap` mapping counts to vectors of elements directly from a vector.

## Installation

Add `count-invert` to your `Cargo.toml`:

```toml
[dependencies]
count-invert = "0.1.0"
```

## Usage
Here's how to use the count-invert crate:

`into_counts`
Convert a vector into a HashMap where the keys are the elements and the values are their counts.

```rust
use count_invert::into_counts;
use std::collections::HashMap;

let vec = vec![1, 2, 2, 3, 3, 3];
let counts: HashMap<i32, usize> = into_counts(vec);
println!("{:?}", counts); // Output: {1: 1, 2: 2, 3: 3}
```

`invert_map`
Invert a HashMap such that the keys become the values and the values become the keys.

```rust
use count_invert::invert_map;
use std::collections::HashMap;

let mut map = HashMap::new();
map.insert(1, 2);
map.insert(2, 2);
map.insert(3, 3);

let inverted: HashMap<usize, Vec<i32>> = invert_map(map);
println!("{:?}", inverted); // Output: {2: [1, 2], 3: [3]}
```

`into_count_map`
Convert a vector into a HashMap where the keys are the counts of elements and the values are vectors of elements with those counts.

```rust
use count_invert::into_count_map;
use std::collections::HashMap;

let vec = vec![1, 2, 2, 3, 3, 3];
let count_map: HashMap<usize, Vec<i32>> = into_count_map(vec);
println!("{:?}", count_map); // Output: {1: [1], 2: [2], 3: [3]}
```

## Contributing
Contributions are welcome! Please submit pull requests or open issues to suggest improvements or report bugs.

## License
This project is licensed under the MIT License. See the LICENSE file for details.
