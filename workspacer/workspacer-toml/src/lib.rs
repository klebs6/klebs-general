// ---------------- [ File: src/lib.rs ]
#[macro_use] mod imports; use imports::*;

x!{cargo_toml_handle}
x!{check_existence}
x!{check_required_fields_for_integrity}
x!{check_required_fields_for_publishing}
x!{check_version_validity_for_integrity}
x!{check_version_validity_for_publishing}
x!{get_package_section}
x!{is_valid_version}
x!{pin_wildcard_deps}
x!{ready_for_cargo_publish}
x!{validate_integrity}
x!{validate_toml}
x!{build_lock_versions}
