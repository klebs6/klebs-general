# `crate-activity`

`crate-activity` is a Rust-based tool for analyzing the usage metrics of your published crates on [crates.io](https://crates.io). It provides insights such as total downloads, average daily downloads, peak daily downloads, and download trends over time. Additionally, it highlights the most downloaded crates over recent intervals and enables caching for efficient data retrieval.

## Features

- **Comprehensive Usage Analysis**:
  - Aggregates and summarizes crate downloads by day.
  - Calculates total downloads, average daily downloads, and peak daily downloads.
  - Identifies trends in download activity: Increasing, Decreasing, or Stable.

- **Multi-Day Activity Summary**:
  - Reports download statistics for individual crates.
  - Highlights top crates for the last 1-day and 3-day intervals.
  - Provides overall statistics such as median daily downloads and per-crate averages.

- **Efficient Data Handling**:
  - Fetches data from the crates.io API.
  - Supports caching of API responses to reduce redundant requests.

- **Customizable Configuration**:
  - Reads the list of crates and user agent settings from a configuration directory (`~/.published-crates` by default).
  - Automatically creates necessary configuration files if they don't exist.

## Installation

Add `crate-activity` to your Rust project using Cargo:

```bash
cargo install crate-activity
```

Alternatively, clone the repository and run it directly:

```bash
git clone https://github.com/your-username/crate-activity.git
cd crate-activity
cargo build --release
./target/release/crate-activity
```

## Usage

Run `crate-activity` to analyze your crate usage data:

```bash
crate-activity
```

By default, the tool looks for a configuration directory at `~/.published-crates`. It expects the following files:

- **`crate_list.txt`**: A list of crate names to analyze, one per line.
- **`user_agent.txt`**: A custom user agent string for API requests.

If these files are missing, the tool will generate defaults.

### Output Example

```
Crate Activity Summary:
  Date Interval (Last 1 Day):  2024-12-08
  Date Interval (Last 3 Days): 2024-12-06

  Total Downloads:                   15,000
  Average Daily Downloads:           5,000.00
  Average Daily Downloads per Crate: 2,500.00
  Median Daily Downloads:            3,000
  Crates Analyzed:                   6

Top Crates (Last 1 Day):
  serde                          3,500 downloads
  tokio                          3,000 downloads

Top Crates (Last 3 Days):
  serde                          8,500 downloads
  tokio                          6,000 downloads
```

## Configuration

### Directory Structure

The configuration directory is located at `~/.published-crates` by default. It should have the following structure:

```
.published-crates/
├── cache/
├── crate_list.txt
└── user_agent.txt
```

### Default Configuration

If the directory or files do not exist, they will be created with default values:

- **`crate_list.txt`**:
  ```txt
  serde
  tokio
  ```
- **`user_agent.txt`**:
  ```txt
  crate-activity-bot/1.0 (contact@example.com)
  ```

### Cache

Cached responses are stored in the `cache/` subdirectory and named using the pattern `<crate_name>_<date>.json`. This allows reusing data from the same day without re-fetching it from the API.

## Development

Clone the repository and explore the code:

```bash
git clone https://github.com/your-username/crate-activity.git
cd crate-activity
```

The main entry point is `src/main.rs`. Core functionality is modularized for ease of maintenance and extension.

### Testing

Run the tests to ensure functionality:

```bash
cargo test
```

### Contributing

Contributions are welcome! If you encounter bugs or have feature suggestions, feel free to open an issue or submit a pull request.

## License

`crate-activity` is licensed under the [MIT License](LICENSE).

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org).
- Inspired by the need to efficiently monitor crate usage on crates.io.

---

Start monitoring your crate usage today with `crate-activity`! 🚀
