// ---------------- [ File: src/lib.rs ]
#![feature(more_qualified_paths)]
#![allow(unused_imports)]
#![allow(unreachable_code)]
#![allow(unused_variables)]

#[macro_use] mod imports; use imports::*;

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/mod.rs"));
}

x!{address_record}
x!{build_all_region_data}
x!{city_name}
x!{cli}
x!{compressed_list}
x!{create_tiny_osm_pbf}
x!{data_access_traits}
x!{data_access}
x!{dmv}
x!{download_and_parse_all_regions}
x!{dump}
x!{errors}
x!{filenames}
x!{find_region_for_file}
x!{house_number_in_any_range}
x!{house_number_ranges}
x!{indexing}
x!{keys}
x!{list_all_addresses_in_pbf_dir}
x!{load_all_cities_for_region}
x!{load_all_streets_for_region}
x!{load_done_regions}
x!{load_house_number_ranges}
x!{house_number_parsing_and_storage}
x!{extract_house_number_range_from_tags}
x!{extract_house_number_range_from_element}
x!{merge_house_number_range}
x!{addresses_from_pbf_file_with_house_numbers}
x!{parse_osm_pbf_and_build_house_number_ranges}
x!{meta_key}
x!{mock}
x!{normalize}
x!{osm_parser}
x!{prefix_transform}
x!{region_data}
x!{regional_records}
x!{regions}
x!{remote_data}
x!{storage}
x!{store_house_number_ranges}
x!{street_name}
x!{traits}
x!{validate_all_addresses}
x!{world_address}
