// ---------------- [ File: workspacer-cli/src/topo.rs ]
crate::ix!();

#[derive(Debug, StructOpt)]
pub enum TopoSubcommand {
    /// Operate on an entire workspace
    Ws {
        /// Path to the workspace
        #[structopt(long = "path")]
        path: PathBuf,

        /// If true, produce a layered ordering (rather than a flat toposort)
        #[structopt(long = "layered")]
        layered: bool,

        /// If true, reverse the final ordering (or layering vector)
        #[structopt(long = "reverse")]
        reverse: bool,

        /// If true, remove crates that fail the filter from the graph entirely
        #[structopt(long = "remove-unwanted")]
        remove_unwanted: bool,

        /// Optional substring to exclude crates that contain it
        #[structopt(long = "exclude-substr", default_value="")]
        exclude_substring: String,
    },

    /// Focus on a particular crate in the workspace
    Focus {
        /// Path to the workspace
        #[structopt(long = "path")]
        path: PathBuf,

        /// Which crate is the focus
        #[structopt(long = "focus")]
        focus_crate: String,

        /// If true, produce layered ordering
        #[structopt(long = "layered")]
        layered: bool,

        /// If true, reverse final ordering/layers
        #[structopt(long = "reverse")]
        reverse: bool,

        /// If true, remove crates that fail the filter from the graph
        #[structopt(long = "remove-unwanted")]
        remove_unwanted: bool,

        /// Exclude crates containing this substring
        #[structopt(long = "exclude-substr", default_value="")]
        exclude_substring: String,
    },

    /// Work on a single crate's internal dependencies
    CrateDeps {
        /// Path to the crate
        #[structopt(long = "crate-path")]
        crate_path: PathBuf,

        /// If true, produce a layered ordering
        #[structopt(long = "layered")]
        layered: bool,

        /// If true, reverse final ordering
        #[structopt(long = "reverse")]
        reverse: bool,

        /// If true, remove crates that fail the filter
        #[structopt(long = "remove-unwanted")]
        remove_unwanted: bool,

        /// Exclude crates containing this substring
        #[structopt(long = "exclude-substr", default_value="")]
        exclude_substring: String,
    },
}

impl TopoSubcommand {
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        match self {
            // -----------------------------------------------------------------
            // 1) Entire workspace
            // -----------------------------------------------------------------
            TopoSubcommand::Ws {
                path,
                layered,
                reverse,
                remove_unwanted,
                exclude_substring,
            } => {
                // Clone needed fields to avoid lifetime issues
                let path_cloned = path.clone();
                let layered_flag = *layered;
                let reverse_flag = *reverse;
                let rm_unwanted = *remove_unwanted;
                let excl_substr = exclude_substring.clone();

                trace!("TopoSubcommand::Ws => path='{}', layered={}, reverse={}, remove_unwanted={}, exclude={}",
                       path_cloned.display(), layered_flag, reverse_flag, rm_unwanted, excl_substr);

                run_with_workspace(Some(path_cloned), /*skip_git_check=*/false, move |ws| {
                    // We move all needed clones into the closure
                    let layered_flag = layered_flag;
                    let reverse_flag = reverse_flag;
                    let rm_unwanted = rm_unwanted;
                    let excl_substr = excl_substr.clone();

                    Box::pin(async move {
                        let filter = build_filter_if_needed(&excl_substr);
                        let mut config_builder = TopologicalSortConfigBuilder::default();
                        config_builder
                            .layering_enabled(layered_flag)
                            .reverse_order(reverse_flag)
                            .remove_unwanted_from_graph(rm_unwanted)
                            .filter_fn(filter);
                        let config = config_builder.build().unwrap();

                        if layered_flag {
                            let layered = ws.layered_topological_order_crate_names(&config).await?;
                            info!("Workspace layering => total {} layers", layered.len());
                            for (i, layer) in layered.iter().enumerate() {
                                println!("Layer {} => {:?}", i, layer);
                            }
                        } else {
                            let sorted = ws.topological_order_crate_names(&config).await?;
                            info!("Workspace flat => sorted crates: {:?}", sorted);
                            for c in sorted {
                                println!("{}", c);
                            }
                        }
                        Ok(())
                    })
                })
                .await
            }

            // -----------------------------------------------------------------
            // 2) Focus => partial workspace up to a single crate
            // -----------------------------------------------------------------
            TopoSubcommand::Focus {
                path,
                focus_crate,
                layered,
                reverse,
                remove_unwanted,
                exclude_substring
            } => {
                let path_cloned = path.clone();
                let focus_cloned = focus_crate.clone();
                let layered_flag = *layered;
                let reverse_flag = *reverse;
                let rm_unwanted = *remove_unwanted;
                let excl_substr = exclude_substring.clone();

                trace!("TopoSubcommand::Focus => path='{}', focus='{}', layered={}, reverse={}",
                       path_cloned.display(), focus_cloned, layered_flag, reverse_flag);

                run_with_workspace(Some(path_cloned), /*skip_git_check=*/false, move |ws| {
                    let layered_flag = layered_flag;
                    let reverse_flag = reverse_flag;
                    let rm_unwanted = rm_unwanted;
                    let excl_substr = excl_substr.clone();
                    let focus_cloned = focus_cloned.clone();

                    Box::pin(async move {
                        let filter = build_filter_if_needed(&excl_substr);
                        let mut config_builder = TopologicalSortConfigBuilder::default();
                        config_builder
                            .layering_enabled(layered_flag)
                            .reverse_order(reverse_flag)
                            .remove_unwanted_from_graph(rm_unwanted)
                            .filter_fn(filter);
                        let config = config_builder.build().unwrap();

                        if layered_flag {
                            let layers = ws.layered_topological_order_upto_crate(&config, &focus_cloned).await?;
                            info!("Focus layered => total {} layers => crate='{}'", layers.len(), focus_cloned);
                            for (i, layer) in layers.iter().enumerate() {
                                println!("Layer {} => {:?}", i, layer);
                            }
                        } else {
                            let partial = ws.topological_order_upto_crate(&config, &focus_cloned).await?;
                            info!("Focus flat => partial => crate='{}': {:?}", focus_cloned, partial);
                            for c in partial {
                                println!("{}", c);
                            }
                        }
                        Ok(())
                    })
                })
                .await
            }

            // -----------------------------------------------------------------
            // 3) CrateDeps => single crate internal deps
            // -----------------------------------------------------------------
            TopoSubcommand::CrateDeps {
                crate_path,
                layered,
                reverse,
                remove_unwanted,
                exclude_substring,
            } => {
                let crate_path_cloned = crate_path.clone();
                let layered_flag = *layered;
                let reverse_flag = *reverse;
                let rm_unwanted = *remove_unwanted;
                let excl_substr = exclude_substring.clone();

                trace!("TopoSubcommand::CrateDeps => crate='{}', layered={}, reverse={}",
                       crate_path_cloned.display(), layered_flag, reverse_flag);

                run_with_crate(crate_path_cloned, /*skip_git_check=*/false, move |handle| {
                    let layered_flag = layered_flag;
                    let reverse_flag = reverse_flag;
                    let rm_unwanted = rm_unwanted;
                    let excl_substr = excl_substr.clone();

                    Box::pin(async move {
                        let filter = build_filter_if_needed(&excl_substr);
                        let mut config_builder = TopologicalSortConfigBuilder::default();
                        config_builder
                            .layering_enabled(layered_flag)
                            .reverse_order(reverse_flag)
                            .remove_unwanted_from_graph(rm_unwanted)
                            .filter_fn(filter);
                        let config = config_builder.build().unwrap();

                        if layered_flag {
                            let layers = handle.layered_topological_order_upto_self(&config).await?;
                            info!("CrateDeps => layered => crate='{}' => {} layers", handle.name(), layers.len());
                            for (i, layer) in layers.iter().enumerate() {
                                println!("Layer {} => {:?}", i, layer);
                            }
                        } else {
                            let sorted = handle.topological_sort_internal_deps(&config).await?;
                            info!("CrateDeps => flat => crate='{}' => {:?}", handle.name(), sorted);
                            for c in sorted {
                                println!("{}", c);
                            }
                        }
                        Ok(())
                    })
                })
                .await
            }
        }
    }
}

/// Builds a filter closure if `exclude_substring` is non-empty.
fn build_filter_if_needed(exclude_substring: &str) -> Option<Arc<dyn Fn(&str)->bool + Send + Sync>> {
    if exclude_substring.is_empty() {
        None
    } else {
        let needle = exclude_substring.to_string();
        Some(Arc::new(move |crate_name: &str| {
            !crate_name.contains(&needle)
        }) as Arc<dyn Fn(&str)->bool + Send + Sync>)
    }
}
