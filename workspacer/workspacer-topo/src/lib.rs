// workspacer-topo/src/lib.rs
//
// This shows how to invert the design so that:
//
//  1) We have a `TopologicalSortConfig` config struct (the "strategy" object).
//  2) The main trait implementations (`BasicTopologicalSort`, `LayeredTopologicalSort`,
//     `FocusCrateTopologicalSort`) live on `Workspace<P, H>` and `CrateHandle`, rather
//     than on the `TopologicalSortConfig` itself. That means you can do:
//
//         let sorter = TopologicalSorterBuilder::default()
//             .reverse_order(true)
//             // etc
//             .build()
//             .unwrap();
//
//         let topo_list = my_workspace.topological_order_crate_names(&sorter).await?;
//
//         let layered = my_workspace.layered_topological_order_crate_names(&sorter).await?;
//
//         let partial = my_workspace.topological_order_upto_crate(&sorter, "focus_crate").await?;
//
//         let partial_handle = my_crate_handle.topological_sort_internal_deps(&sorter).await?;
//
// The `TopologicalSortConfig` is just a config (fields: `reverse_order`, `filter_fn`, etc),
// while each trait method is implemented *on* either `Workspace<P, H>` or `CrateHandle`,
// with the `config: &TopologicalSortConfig` guiding how we do filtering, layering, reversing, etc.
//
// ---------------------------------------------------------------------------
#[macro_use] mod imports; use imports::*;

x!{basic_topologcal_sort}
x!{config}
x!{focus_crate_topological_sort}
x!{layered_subgraph_internal}
x!{layered_topologcal_sort}
x!{topological_sort_internal_deps}
x!{traits}
