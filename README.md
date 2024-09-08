This is a workspace for crates which are broadly useful across a variety of projects. 

Here we have the following crates:

### `backoff-macro`
- Mark an async function with #[backoff] to get the default ExponentialBackoff behavior. (tokio compatible)

### `count-invert`
- A Rust crate providing utility functions for counting elements in a vector and inverting a HashMap based on those counts.

### `disable-macro`
- This simple crate lets us disable a block of code with an attribute #[disable].

### `error-tree`
- This crate let's us use the `error_tree!` proc macro for ergonomic error hierarchy definition

### `export-magic`
- A crate to simplify module management and re-exportation using macros.

### `find-matching-bracket`
- Finds the matching closing bracket for a given opening bracket in a string. Supports curly braces, square brackets, and parentheses. This crate is useful for parsing code, validating expressions, and more.

### `fs-shorthand`
- Provides a set of filesystem utility functions.

### `gpt-batch-scribe`
- contains the GptBatchAPIRequest struct which helps create gpt4 batch requests.

### `language-enum`
- A robust enum representing languages for global and regional applications.

### `named-item`
- A crate providing traits for managing named items, including support for aliases, name history, validation, and more.

### `pbx`
- pbx is a Rust crate providing convenient macros and utility functions for creating and managing boxed, atomic reference-counted, and default-initialized values. The crate simplifies common patterns in Rust, especially useful in asynchronous programming, concurrent access, and interfacing with C libraries.

### `renew-traits`
- A collection of utility traits for initializing, filling, and managing collections or data structures.

### `resume-generator`
- does what it says. quick way to generate a latex resume

### `static-or-heap-string`
- An enum type for handling both static and heap-allocated strings.

### `str-shorthand`
- A Rust crate that provides utility functions for string manipulation. Includes a function to bisect a string into two halves, handling multi-byte UTF-8 characters correctly.

### `traced-test`
- this crate lets us use #[traced_test] to automatically configure sane default tracing for a rust test

### `tracing-setup`
- this crate helps us configure tracing for a rust project. It is designed to be used with the `traced-test` crate

