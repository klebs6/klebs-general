
// file-downloader-derive/tests/ui/ok_unit_variant_link.rs
use file_downloader::FileDownloader;
use file_downloader_derive::FileDownloader;

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

