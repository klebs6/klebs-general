This is a proc-macro crate for the FileDownloader trait found in the file-downloader crate.

We use it like this:

```rust
#[derive(FileDownloader)]
enum MyEnum {
    #[download_link("https://example.com/data1.pbf")]
    VariantOne,

    // Could do multiple unit variants, each with its own link
    #[download_link("https://example.com/data2.pbf")]
    VariantTwo,
}

fn main() {
    let v1 = MyEnum::VariantOne;
    assert_eq!(v1.download_link(), "https://example.com/data1.pbf");

    let v2 = MyEnum::VariantTwo;
    assert_eq!(v2.download_link(), "https://example.com/data2.pbf");
}
```

Then, we can do:

```rust
fn main() -> Result<(),DownloadError> {
    let v1 = MyEnum::VariantOne;
    assert_eq!(v1.download_link(), "https://example.com/data1.pbf");
    v1.find_file_locally_or_download_into("target_path").await?;
    Ok(())
}
```
