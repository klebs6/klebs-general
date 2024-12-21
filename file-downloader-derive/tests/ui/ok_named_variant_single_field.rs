
// file-downloader-derive/tests/ui/ok_named_variant_single_field.rs
use file_downloader::FileDownloader;
use file_downloader_derive::FileDownloader;

struct NamedDownloader;
impl FileDownloader for NamedDownloader {
    fn download_link(&self) -> &'static str {
        "https://example.com/named.pbf"
    }
}

#[derive(FileDownloader)]
enum MyEnum {
    NamedVariant {
        inner: NamedDownloader
    }
}

fn main() {
    let v = MyEnum::NamedVariant { inner: NamedDownloader };
    assert_eq!(v.download_link(), "https://example.com/named.pbf");
}

