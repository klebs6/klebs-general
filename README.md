This is a workspace for crates which are broadly useful across a variety of projects. 

Here we have the following crates:

# `backoff-macro`
- Mark an async function with #[backoff] to get the default ExponentialBackoff behavior. (tokio compatible)

# `disable-macro`
- This simple crate lets us disable a block of code with an attribute #[disable].

# `error-tree`
- This crate let's us use the `error_tree!` proc macro for ergonomic error hierarchy definition

# `export-magic`
- A crate to simplify module management and re-exportation using macros.

# `traced-test`
- this crate lets us use #[traced_test] to automatically configure sane default tracing for a rust test

# `tracing-setup`
- This crate helps us configure tracing for a rust project. It is designed to be used with the `traced-test` crate
