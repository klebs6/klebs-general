# Gather-All-Code-From-Crates

`gather-all-code-from-crates` is a Rust crate designed to extract, filter, and reconstruct code elements from Rust projects. It provides a flexible and configurable toolset for analyzing and processing Abstract Syntax Trees (ASTs) of Rust code, with options to include or exclude specific elements based on user-defined criteria.

## Disclaimer
Version 0.1.0 is a quick sketch. 

It needs a refactor, a thorough testing, and is
bound to have some bugs. 

That said, the basic functionality is currently
*decent* and mostly working. 

Pull requests are most welcome.


## Features

- **AST Filtering**: Extract structs, enums, functions, and other items from Rust ASTs.
- **Customizable Criteria**: Filter items by visibility, test inclusion, file paths, and more.
- **Configuration Options**:
  - Global configuration through JSON files.
  - CLI arguments for runtime customization.
- **Reconstruction**: Rebuild filtered code snippets with options to include or omit function bodies and documentation comments.
- **Error Handling**: Comprehensive error handling for invalid inputs, configuration issues, and more.

## Installation

Add this crate to your `Cargo.toml`:

```toml
[dependencies]
gather-all-code-from-crates = "0.1.0"
```

## Usage

### Basic Example

Run the tool using the CLI to gather and filter code from crates:

```bash
cargo run -- --crate path/to/crate --include-tests --omit-private
```

### Programmatic API

#### Loading Global Configuration

```rust
use gather_all_code_from_crates::{load_global_config, GlobalConfig};

let config = load_global_config()?;
```

#### Filtering AST Items

```rust
use gather_all_code_from_crates::{process_crate_directory, AstFilterCriteria};
use std::path::PathBuf;

let criteria = AstFilterCriteria::default();
let crate_path = PathBuf::from("path/to/crate");
let result = process_crate_directory(&crate_path, &criteria)?;
println!("Filtered Code:\n{}", result);
```

#### Building Effective Config

```rust
use gather_all_code_from_crates::build_effective_config_from_cli;

let effective_config = build_effective_config_from_cli()?;
println!("Effective Config: {:?}", effective_config);
```

## CLI Options

- `--crate <path>`: Specify one or more crate directories to scan. Defaults to the current directory if not provided.
- `--include-tests`: Include test code in the output.
- `--omit-private`: Exclude private functions and items.
- `--omit-bodies`: Exclude function bodies in the output.
- `--single-test <name>`: Include only a single test block by name.
- `--single-function <name>`: Include only a single function body by name.
- `--exclude-file <path>`: Exclude specific files by relative path.
- `--exclude-main-file`: Exclude main files such as `src/lib.rs` or `src/main.rs`.
- `--remove-doc-comments`: Remove documentation comments from the output.

## Configuration

Global configuration is loaded from a JSON file located at `~/.gather-all-code-from-crates`. Example:

```json
{
  "project_overrides": {
    "project_a": {
      "include_tests": true,
      "omit_private": false
    }
  },
  "default_include_tests": true,
  "default_omit_bodies": false,
  "extra_flags": 0
}
```

## Error Handling

The crate defines custom error types (`AppError`) to handle different scenarios, including:
- Missing configuration or data.
- Invalid arguments.
- File I/O errors.
- AST parsing issues.

## Contributing

Contributions are welcome! Please submit a pull request or open an issue to discuss any changes or features.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
