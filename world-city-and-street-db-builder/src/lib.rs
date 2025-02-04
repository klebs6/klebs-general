// ---------------- [ File: src/lib.rs ]
#![feature(more_qualified_paths)]
#![allow(unused_imports)]
#![allow(unreachable_code)]
#![allow(unused_variables)]

#[macro_use] mod imports; use imports::*;

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/mod.rs"));
}

x!{addresses_from_pbf_file_with_house_numbers}
x!{address_record}
x!{address_record_from_element_and_country}
x!{address_record_from_tags}
x!{build_all_region_data}
x!{build_world_address}
x!{city_name}
x!{cli}
x!{collect_tags}
x!{compressed_list}
x!{create_tiny_osm_pbf}
x!{data_access}
x!{data_access_traits}
x!{dmv}
x!{download_and_parse_all_regions}
x!{download_and_parse_regions}
x!{dump}
x!{errors}
x!{expected_filename_for_region}
x!{extract_house_number_range_from_element}
x!{extract_house_number_range_from_tags}
x!{filenames}
x!{filenames_match}
x!{find_region_for_file}
x!{house_number_in_any_range}
x!{house_number_parsing_and_storage}
x!{house_number_ranges}
x!{indexing}
x!{keys}
x!{list_all_addresses_in_pbf_dir}
x!{load_all_cities_for_region}
x!{load_all_streets_for_region}
x!{load_done_regions}
x!{load_house_number_ranges}
x!{merge_house_number_range}
x!{meta_key}
x!{mock}
x!{normalize}
x!{obtain_pbf_file_for_region}
x!{open_osm_pbf_reader}
x!{parse_and_aggregate_osm}
x!{parse_osm_pbf_and_build_house_number_ranges}
x!{prefix_transform}
x!{process_osm_element}
x!{region_data}
x!{regional_records}
x!{regions}
x!{remote_data}
x!{storage}
x!{store_aggregator_results}
x!{store_house_number_ranges}
x!{street_name}
x!{strip_leading_dot_slash}
x!{traits}
x!{validate_all_addresses}
x!{validate_pbf_filename}
x!{world_address}
x!{write_be_u32}
