// ---------------- [ File: workspacer-show/src/build_filtered_string.rs ]
crate::ix!();

impl ShowFlags {

    /// Builds the final string from a `ConsolidatedCrateInterface`, applying the post-filters from `ShowFlags`.
    #[tracing::instrument(level = "trace", skip(cci, crate_name))]
    pub fn build_filtered_string(
        &self,
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
            if *self.show_items_with_no_data() {
                return "<no-data-for-crate>\n".to_string();
            } else {
                return String::new();
            }
        }

        // If we want to group by file, do it:
        if *self.group_by_file() {
            return self.build_filtered_grouped_by_file_string(cci);
        }

        // If the user wants only certain categories, we do that.
        // (Replicates the old show sub-filters.)
        if *self.just_fns() {
            return self.build_output_for_items(cci.fns());
        }
        if *self.just_impls() {
            return self.build_output_for_items(cci.impls());
        }
        if *self.just_traits() {
            return self.build_output_for_items(cci.traits());
        }
        if *self.just_enums() {
            return self.build_output_for_items(cci.enums());
        }
        if *self.just_structs() {
            return self.build_output_for_items(cci.structs());
        }
        if *self.just_aliases() {
            return self.build_output_for_items(cci.type_aliases());
        }
        if *self.just_adts() {
            let mut combined = Vec::new();
            for e in cci.enums() {
                combined.push(format!("{}", e));
            }
            for s in cci.structs() {
                combined.push(format!("{}", s));
            }
            if combined.is_empty() && *self.show_items_with_no_data() {
                return "<no-data-for-crate>\n".to_string();
            }
            return join_with_blank_line(combined);
        }
        if *self.just_macros() {
            return self.build_output_for_items(cci.macros());
        }

        // Otherwise, print the entire consolidated interface in one go
        let all = format!("{}", cci);
        if all.trim().is_empty() && *self.show_items_with_no_data() {
            return "<no-data-for-crate>\n".to_string();
        }
        all
    }
}
