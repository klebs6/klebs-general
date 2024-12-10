# `crate-activity`

# Crate Activity Analyzer

**Crate Activity Analyzer** is a command-line tool designed to analyze and visualize the activity patterns of various Rust crates over time. It fetches usage data (downloads), cleans it by detecting and handling outliers, and performs advanced statistical and network-based analyses such as correlation analysis, Principal Component Analysis (PCA), hierarchical clustering, and correlation network exploration.

This tool aims to help developers and researchers understand how crates evolve, identify underlying usage patterns, cluster crates by similar activity, and detect anomalies or spikes that might skew correlations.

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

- **Correlation Analysis:**  
  Compute pairwise correlations between crates based on their daily download patterns. Focus on crates with strong correlations to identify related ecosystems or usage trends.

- **PCA Analysis:**  
  Reduce dimensionality and reveal underlying factors that explain most of the variance in crate usage data.

- **Hierarchical Clustering:**  
  Perform single-linkage hierarchical clustering to group crates into dendrograms, revealing how crates cluster together by similarity in their download activity.

- **Correlation Network Analysis:**  
  Build a network graph of crates as nodes and strong correlations as edges. Analyze communities of crates using Girvan–Newman or examine betweenness centrality to find critical "bridge" crates.

- **Time-Lagged Correlations:**  
  Explore correlations with potential time shifts to see if one crate's activity leads or lags behind another, potentially uncovering cause-effect or dependency relationships.

- **Outlier Detection and Handling:**  
  Identify and remove or downweight anomalous spikes in download data using a MAD-based robust outlier detection approach. Helps ensure that rare, extreme spikes don't distort the overall patterns.

- **Fully Configurable via CLI:**  
  Choose which analyses to run individually or run them all at once with `--all`.  
  Adjust thresholds (like correlation network threshold, outlier z-threshold) and control outlier handling (disable, remove, or downweight them).

## Installation

Ensure you have Rust and Cargo installed. Then:

```bash
cargo install crate-activity
```

## Usage

To run the analysis, simply call:

```bash
crate-activity
```

By default, the tool looks for a configuration directory at `~/.published-crates`. It expects the following files:

- **`crate_list.txt`**: A list of crate names to analyze, one per line.
- **`user_agent.txt`**: A custom user agent string for API requests.

If these files are missing, the tool will generate defaults.

### Common CLI Flags

- `--all`  
  Enable all analyses at once (correlations, PCA, hierarchical clustering, network analysis, etc.).

- `--show-correlations` (or `-c`)  
  Display correlation analysis results.

- `--perform-pca` (or `-p`)  
  Run PCA on the download data.

- `--perform-hierarchical-clustering` (or `-h`)  
  Compute hierarchical clustering and print a dendrogram.

- `--correlation-network` (or `-n`)  
  Build and analyze a correlation network graph.

- `--print-summary` (or `-s`)  
  Print a summary of the network graph (number of nodes, edges, communities).

- `--time-lag-correlations` (or `-t`)  
  Compute and display time-lagged correlations with a given `--max-lag`.

- `--outlier-z-threshold <float>`  
  Set the z-score threshold for detecting outliers. Higher values yield fewer outliers.

- `--downweight-outliers`  
  Instead of removing outliers, downweight them by `--outlier-weight` factor.

- `--outlier-weight <float>`  
  Factor by which to multiply outliers if downweighting them.

- `--disable-outlier-handling`  
  Completely skip outlier detection and use raw data.

### Example Commands

1. **Run Everything:**  
   ```bash
   crate-activity --all
   ```
   This enables correlation analysis, PCA, hierarchical clustering, network analysis, print summary, and time-lag correlations.

2. **Focus on Correlation Analysis Only:**  
   ```bash
   crate-activity --show-correlations
   ```
   
3. **Perform PCA and Hierarchical Clustering with Higher Outlier Threshold:**  
   ```bash
   crate-activity --perform-pca --perform-hierarchical-clustering --outlier-z-threshold 6.0
   ```
   
4. **Disable Outlier Handling and Build a Correlation Network:**  
   ```bash
   crate-activity --disable-outlier-handling --correlation-network --print-summary
   ```

## Configuration

- **Crate List:**  
  By default, reads `~/.published-crates/crate_list.txt`.  
  If missing, it warns and uses a default set (like `serde`, `tokio`).

- **User Agent:**  
  Tries to read `~/.published-crates/user_agent.txt`. If missing, uses a default user agent.

- **Caching:**  
  Responses are cached in `~/.published-crates/cache` to speed up repeated runs.

## Testing

Run tests with:

```bash
cargo test
```

Tests verify data alignment, correlation computation, PCA, clustering, and outlier handling. If you encounter test failures due to outlier handling in tests, remember that the code disables outliers by default in test builds (or adjust test scenarios accordingly).

## Limitations and Future Work

- The current outlier detection method (MAD-based) and thresholds might need domain-specific tuning.
- PCA and clustering are based on Pearson correlations, which assume linear relationships.
- No built-in visualization beyond terminal output. Users may export results and visualize them with external tools.


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

