This is a proc-macro crate for the FileDownloader trait found in the file-downloader crate.

We use it like this:

```rust
#[derive(OsmPbfFileDownloader)]
enum MyEnum {
    #[geofabrik(spain="madrid-latest.osm.pbf")]
    VariantOne,

    // Could do multiple unit variants, each with its own link
    #[geofabrik(poland="mazowieckie-latest.osm.pbf")]
    VariantTwo,
}
```

Then, we can do:

```rust
#[tokio::main]
async fn main() -> Result<(),Box<dyn Error>> {

    let v1 = MyEnum::VariantOne;
    assert_eq!(v1.download_link(), "https://download.geofabrik.de/europe/spain/madrid-latest.osm.pbf");

    let v2 = MyEnum::VariantTwo;
    assert_eq!(v2.download_link(), "https://download.geofabrik.de/europe/poland/mazowieckie-latest.osm.pbf");

    v1.find_file_locally_or_download_into("target_path").await?;
    Ok(())
}
```

This crate is a shortcut, similar to the `FileDownloader` proc macro, given the
task of downloading OSM (open streetmap) PBF files follows a common pattern.

It is designed to reduce boilerplate.
