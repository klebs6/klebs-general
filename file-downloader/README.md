# File Downloader

`file_downloader` is a Rust library designed to help you efficiently download and verify possibly md5 checksummed files. It offers functionality to:

- Download files from a specified URL.
- Verify downloaded files against provided MD5 checksums.
- Store files locally and skip downloads if a verified local copy is available.

## Features

- **Checksum Verification**: Ensures the integrity of downloaded files by verifying their MD5 checksums.
- **Local Caching**: Skips re-downloading files if a checksum-matching local copy is found.
- **Clean Abstraction**: The `FileDownloader` trait provides a simple interface for downloading files, so you only need to implement the `download` method to leverage all of this crate’s functionality.

## Example

```rust
use file_downloader::{FileDownloader, PbfDownloadError};
use std::path::PathBuf;
use async_trait::async_trait;

struct PlanetDownloader;

#[async_trait]
impl FileDownloader for PlanetDownloader {

    fn download_link(&self) -> &str {
        "https://planet.openstreetmap.org/pbf/planet-latest.osm.pbf"
    }
}

#[tokio::main]
async fn main() -> Result<(), PbfDownloadError> {
    let downloader = PlanetDownloader;
    let local_path = downloader.find_file_locally_or_download("./pbf_cache").await?;
    println!("file available at: {:?}", local_path);
    Ok(())
}
```

## Testing

This crate includes a comprehensive test suite that covers:

- MD5 checksum computation and verification
- Filename manipulation functions
- HTTP download handling, including mocking of HTTP responses
- Integration tests for end-to-end downloading and verification

You can run tests with:
```bash
cargo test
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
