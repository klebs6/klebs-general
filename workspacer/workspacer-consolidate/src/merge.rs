// ---------------- [ File: workspacer-consolidate/src/merge.rs ]
crate::ix!();

/// A local helper to merge one crate interface into another in-place.
#[instrument(level = "trace", skip_all)]
pub fn merge_in_place(base: &mut ConsolidatedCrateInterface, addition: &ConsolidatedCrateInterface) {
    trace!("Merging 'addition' into 'base' in-place");
    base.enums_mut().extend_from_slice(addition.enums());
    base.structs_mut().extend_from_slice(addition.structs());
    base.traits_mut().extend_from_slice(addition.traits());
    base.impls_mut().extend_from_slice(addition.impls());
    base.fns_mut().extend_from_slice(addition.fns());
    base.macros_mut().extend_from_slice(addition.macros());
    base.type_aliases_mut().extend_from_slice(addition.type_aliases());
    // If you have other fields (consts, statics, etc.), extend them here as well.
    trace!("Done merging crate interface data");
}
