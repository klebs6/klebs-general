
// file-downloader-derive/tests/ui/ok_tuple_variant_single_field.rs
use file_downloader::FileDownloader;
use file_downloader_derive::FileDownloader;

struct UnderlyingDownloader;

impl FileDownloader for UnderlyingDownloader {
    fn download_link(&self) -> &'static str {
        "https://example.com/underlying.pbf"
    }
}

#[derive(FileDownloader)]
enum MyEnum {
    // We'll delegate to the single field's download_link()
    TupleVariant(UnderlyingDownloader),
}

fn main() {
    let v = MyEnum::TupleVariant(UnderlyingDownloader);
    assert_eq!(v.download_link(), "https://example.com/underlying.pbf");
}

