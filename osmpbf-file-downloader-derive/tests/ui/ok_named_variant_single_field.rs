// osmpbf-file-downloader-derive/tests/ui/ok_named_variant_single_field.rs
use file_downloader::*;
use osmpbf_file_downloader_derive::OsmPbfFileDownloader;

struct NamedDownloader;
impl FileDownloader for   NamedDownloader {}
impl Md5DownloadLink      for NamedDownloader {}
impl DownloadLink         for NamedDownloader {
    fn download_link(&self) -> &'static str {
        "https://example.com/named.pbf"
    }
}

#[derive(OsmPbfFileDownloader)]
enum MyEnum {
    NamedVariant {
        inner: NamedDownloader
    }
}

fn main() {
    let v = MyEnum::NamedVariant { inner: NamedDownloader };
    assert_eq!(v.download_link(), "https://example.com/named.pbf");
}
