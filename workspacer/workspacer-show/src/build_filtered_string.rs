crate::ix!();

/// Builds the final string from a `ConsolidatedCrateInterface`, applying the post-filters from `ShowFlags`.
#[tracing::instrument(level = "trace", skip(options, cci, crate_name))]
pub fn build_filtered_string(
    options: &ShowFlags,
    cci: &ConsolidatedCrateInterface,
    crate_name: &str,
) -> String {
    trace!("Applying post-filters in build_filtered_string for crate={}", crate_name);

    // If the entire crate is empty (no items in cci), we can short-circuit.
    let crate_is_empty = cci.fns().is_empty()
        && cci.structs().is_empty()
        && cci.enums().is_empty()
        && cci.traits().is_empty()
        && cci.type_aliases().is_empty()
        && cci.macros().is_empty()
        && cci.impls().is_empty()
        && cci.modules().is_empty();

    if crate_is_empty {
        if options.show_items_with_no_data() {
            return "<no-data-for-crate>\n".to_string();
        } else {
            return String::new();
        }
    }

    // If we want to group by file, do it:
    if options.group_by_file() {
        return build_filtered_grouped_by_file_string(options, cci);
    }

    // If the user wants only certain categories, we do that.
    // (Replicates the old show sub-filters.)
    if options.just_fns() {
        return build_output_for_items(cci.fns(), options);
    }
    if options.just_impls() {
        return build_output_for_items(cci.impls(), options);
    }
    if options.just_traits() {
        return build_output_for_items(cci.traits(), options);
    }
    if options.just_enums() {
        return build_output_for_items(cci.enums(), options);
    }
    if options.just_structs() {
        return build_output_for_items(cci.structs(), options);
    }
    if options.just_aliases() {
        return build_output_for_items(cci.type_aliases(), options);
    }
    if options.just_adts() {
        let mut combined = Vec::new();
        for e in cci.enums() {
            combined.push(format!("{}", e));
        }
        for s in cci.structs() {
            combined.push(format!("{}", s));
        }
        if combined.is_empty() && options.show_items_with_no_data() {
            return "<no-data-for-crate>\n".to_string();
        }
        return join_with_blank_line(combined);
    }
    if options.just_macros() {
        return build_output_for_items(cci.macros(), options);
    }

    // Otherwise, print the entire consolidated interface in one go
    let all = format!("{}", cci);
    if all.trim().is_empty() && options.show_items_with_no_data() {
        return "<no-data-for-crate>\n".to_string();
    }
    all
}
