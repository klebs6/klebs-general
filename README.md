This is a workspace for crates which are broadly useful across a variety of projects. 

Here we have the following crates:

### `ai-descriptor-derive`
- Provides a proc macro we can use to annotate enums for facilitating itemization and description using an AI model

### `ai-descriptor-trait`
- Provides an interface for the proc macro we can use to annotate enums for facilitating itemization and description using an AI model

### `ai-descriptor`
- Provides a proc macro we can use to annotate enums for facilitating itemization and description using an AI model

### `backoff-macro`
- Mark an async function with #[backoff] to get the default ExponentialBackoff behavior. (tokio compatible)

### `birthday-struct`
- A crate for representing and manipulating birthdays, with support for time zones, age calculations, and integration with zodiac signs.

### `count-invert`
- A Rust crate providing utility functions for counting elements in a vector and inverting a HashMap based on those counts.

### `country`
- A Rust library providing a single source of truth for country enumeration and their associated ISO 3166-1 alpha-2 and alpha-3 codes

### `crate-activity`
- This crate provides a way to monitor the usage for a set of crates.io crates

### `disable-macro`
- This simple crate lets us disable a block of code with an attribute #[disable].

### `error-tree`
- This crate let's us use the `error_tree!` proc macro for ergonomic error hierarchy definition

### `export-magic`
- A crate to simplify module management and re-exportation using macros.

### `find-matching-bracket`
- Finds the matching closing bracket for a given opening bracket in a string. Supports curly braces, square brackets, and parentheses. This crate is useful for parsing code, validating expressions, and more.

### `form-of-joke-humor`
- A Rust crate for representing and working with various forms of jokes and humor.

### `form-of-wordplay`
- A Rust crate for representing and working with various forms of wordplay.

### `fs-shorthand`
- Provides a set of filesystem utility functions.

### `gather-all-code-from-crates`
- a Rust crate designed to extract, filter, and reconstruct code elements from Rust projects. It provides a flexible and configurable toolset for analyzing and processing Abstract Syntax Trees (ASTs) of Rust code, with options to include or exclude specific elements based on user-defined criteria.

### `gpt-batch-scribe`
- contains the GptBatchAPIRequest struct which helps create gpt4 batch requests.

### `json-repair`
- A well tested crate for repairing malformed JSON strings and repairing/parsing them into valid JSON values

### `language-enum`
- A robust enum representing languages for global and regional applications.

### `lyrical-meter`
- A Rust crate for representing and working with various poetic meters.

### `month-and-season`
- Typed enumerations and utilities for months and meteorological seasons.

### `name-and-title`
- A Rust library for representing personal names with optional titles, middle names/initials, and convenience macros.

### `named-item`
- A crate providing traits for managing named items, including support for aliases, name history, validation, and more.

### `pbx`
- pbx is a Rust crate providing convenient macros and utility functions for creating and managing boxed, atomic reference-counted, and default-initialized values. The crate simplifies common patterns in Rust, especially useful in asynchronous programming, concurrent access, and interfacing with C libraries.

### `plural-derive`
- This crate contains the derive macro `Plural` we use with the `PluralDisplay` trait

### `plural-trait`
- This crate contains the trait we use with the plural-derive proc macro

### `postal-code`
- A robust, production-grade Rust library for validating international postal codes.

### `rand-construct`
- Encapsulates the random-constructible and random-constructible-derive crates which are used for creating random instances of data structures with weighted probabilities

### `random-constructible-derive`
- Provides a derive macro for the random-constructible crate which is used for creating random instances of enums with weighted probabilities

### `random-constructible`
- Provides a trait for creating random instances of enums with weighted probabilities

### `renew-traits`
- A collection of utility traits for initializing, filling, and managing collections or data structures.

### `resume-generator`
- does what it says. quick way to generate a latex resume

### `rhyme-type`
- A crate for representing and generating different types of rhymes.

### `scan-crate-for-typedefs`
- simple crate -- lets us scan crate(s) for locally defined structs, enums, types, fns, and traits

### `static-or-heap-string`
- An enum type for handling both static and heap-allocated strings.

### `str-shorthand`
- A Rust crate that provides utility functions for string manipulation. Includes a function to bisect a string into two halves, handling multi-byte UTF-8 characters correctly.

### `structured-language-form`
- A Rust crate for representing and working with various poetic meters.

### `traced-test`
- this crate lets us use #[traced_test] to automatically configure sane default tracing for a rust test

### `tracing-setup`
- this crate helps us configure tracing for a rust project. It is designed to be used with the `traced-test` crate

### `variant-builder-macro`
- This crate gives us the VariantBuider proc macro which can be used to streamline creting an enum from wrapping variants each using the builder pattern.

### `workspacer`
- A Rust library for managing and validating workspaces and crates, with support for test coverage, circular dependency detection, and publishing readiness.

### `zodiac-sign`
- A Rust library enumerating zodiac signs and providing date-based lookups and conversions.

