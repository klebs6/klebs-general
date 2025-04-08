// ---------------- [ File: workspacer-cli/src/topo.rs ]
crate::ix!();

#[derive(Getters,Debug, StructOpt)]
#[getset(get="pub")]
pub struct TopoSubcommand {
    /// Path to the workspace directory (required unless you're in the workspace root).
    #[structopt(long = "path")]
    workspace_path: Option<PathBuf>,

    /// Optional crate name for either a "focus" subgraph or "internal deps" mode
    #[structopt(long = "crate")]
    crate_name: Option<String>,

    /// If true, produce a layered ordering rather than a flat toposort
    #[structopt(long = "layered")]
    layered: bool,

    /// If true, reverse the final ordering (or layering vector)
    #[structopt(long = "reverse")]
    reverse: bool,

    /// If true, remove crates that fail the filter from the graph entirely
    #[structopt(long = "remove-unwanted")]
    remove_unwanted: bool,

    /// Optional substring to exclude crates whose names contain it
    #[structopt(long = "exclude-substr", default_value="")]
    exclude_substring: String,

    /// If true, include external (third-party) crates in workspace/focus queries
    #[structopt(long = "include-externals")]
    include_externals: bool,

    /// If true, interpret `--crate` as single-crate "internal dependencies" mode
    #[structopt(long = "internal")]
    internal_mode: bool,
}

impl TopoSubcommand {
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        // Branching logic:

        // 1) If user provided `--crate <NAME>`:
        if let Some(ref c_name) = self.crate_name {
            // A) If `--internal`, do single crate internal deps
            if self.internal_mode {
                self.run_single_crate_internal(c_name).await
            }
            // B) Else do partial subgraph "Focus" (ancestors up to crate)
            else {
                self.run_focus_workspace(c_name).await
            }
        }
        // 2) Otherwise => entire workspace
        else {
            self.run_entire_workspace().await
        }
    }

    // -----------------------------------------------------
    // (A) Single Crate Internal Dependencies
    // -----------------------------------------------------
    async fn run_single_crate_internal(&self, crate_name: &str) -> Result<(), WorkspaceError> {
        trace!(
            "TopoSubcommand::run_single_crate_internal => crate='{}', layered={}, reverse={}",
            crate_name, self.layered, self.reverse
        );
        // We'll need a workspace path to locate that crate, or we can attempt to find it
        // in the current working directory. Let's require `--path`.
        let ws_path = self.get_workspace_path("single-crate internal deps")?;

        let layered:         bool = *self.layered();
        let reverse:         bool = *self.reverse();
        let remove_unwanted: bool = *self.remove_unwanted();
        let exclude_substring = self.exclude_substring().to_string();

        let c_name_cloned = crate_name.to_string();

        // 1) Load the workspace
        run_with_workspace(Some(ws_path), /*skip_git_check=*/true, move |ws| {
            // clone relevant fields
            let layered_flag = layered;
            let reverse_flag = reverse;
            let rm_unwanted = remove_unwanted;
            let excl_substr = exclude_substring.clone();

            Box::pin(async move {
                // 2) Attempt to find that crate's handle
                let maybe_crate_arc = ws.find_crate_by_name(&c_name_cloned).await;
                let crate_arc = match maybe_crate_arc {
                    Some(arc) => arc,
                    None => {
                        let msg = format!("No crate named '{}' found in workspace", c_name_cloned);
                        error!("{}", msg);
                        return Err(WorkspaceError::CrateError(CrateError::CrateNotFoundInWorkspace {
                            crate_name: c_name_cloned
                        }));
                    }
                };
                let handle = crate_arc.lock().await;

                // 3) Build the topological config
                let final_filter = if excl_substr.is_empty() {
                    None
                } else {
                    Some(Arc::new(move |nm: &str| !nm.contains(&excl_substr))
                         as Arc<dyn Fn(&str)->bool + Send + Sync>)
                };

                let mut config_builder = TopologicalSortConfigBuilder::default();
                config_builder
                    .layering_enabled(layered_flag)
                    .reverse_order(reverse_flag)
                    .remove_unwanted_from_graph(rm_unwanted)
                    .filter_fn(final_filter);
                let config = config_builder.build().unwrap();

                // 4) Single-crate internal: layered or flat
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

    // -----------------------------------------------------
    // (B) Focus => partial workspace
    // -----------------------------------------------------
    async fn run_focus_workspace(&self, crate_name: &str) -> Result<(), WorkspaceError> {
        trace!(
            "TopoSubcommand::run_focus_workspace => crate='{}', layered={}, reverse={}, externals={}",
            crate_name, self.layered, self.reverse, self.include_externals
        );
        let ws_path = self.get_workspace_path("focus subgraph")?;

        // clone flags
        let layered_flag = self.layered;
        let reverse_flag = self.reverse;
        let rm_unwanted = self.remove_unwanted;
        let excl_substr = self.exclude_substring.clone();
        let show_3p = self.include_externals;
        let focus_crate = crate_name.to_string();

        run_with_workspace(Some(ws_path), /*skip_git_check=*/true, move |ws| {
            Box::pin(async move {
                // gather known crates
                let crate_arcs = ws.crates();
                let mut known_set = HashSet::new();
                for c_arc in crate_arcs {
                    let locked = c_arc.lock().await;
                    known_set.insert(locked.name().to_string());
                }

                let final_filter = Arc::new(move |nm: &str| {
                    // skip externals if !show_3p
                    if !show_3p && !known_set.contains(nm) {
                        return false;
                    }
                    if !excl_substr.is_empty() && nm.contains(&excl_substr) {
                        return false;
                    }
                    true
                }) as Arc<dyn Fn(&str)->bool + Send + Sync>;

                let mut config_builder = TopologicalSortConfigBuilder::default();
                config_builder
                    .layering_enabled(layered_flag)
                    .reverse_order(reverse_flag)
                    .remove_unwanted_from_graph(rm_unwanted)
                    .filter_fn(Some(final_filter));
                let config = config_builder.build().unwrap();

                if layered_flag {
                    let layers = ws.layered_topological_order_upto_crate(&config, &focus_crate).await?;
                    info!("Focus layered => total {} layers => crate='{}'", layers.len(), focus_crate);
                    for (i, layer) in layers.iter().enumerate() {
                        println!("Layer {} => {:?}", i, layer);
                    }
                } else {
                    let partial = ws.topological_order_upto_crate(&config, &focus_crate).await?;
                    info!("Focus flat => partial => crate='{}': {:?}", focus_crate, partial);
                    for c in partial {
                        println!("{}", c);
                    }
                }
                Ok(())
            })
        })
        .await
    }

    // -----------------------------------------------------
    // (C) Entire workspace
    // -----------------------------------------------------
    async fn run_entire_workspace(&self) -> Result<(), WorkspaceError> {
        trace!(
            "TopoSubcommand::run_entire_workspace => layered={}, reverse={}, externals={}",
            self.layered, self.reverse, self.include_externals
        );
        let ws_path = self.get_workspace_path("workspace-level topo")?;

        let layered_flag = self.layered;
        let reverse_flag = self.reverse;
        let rm_unwanted = self.remove_unwanted;
        let excl_substr = self.exclude_substring.clone();
        let show_3p = self.include_externals;

        run_with_workspace(Some(ws_path), /*skip_git_check=*/true, move |ws| {
            Box::pin(async move {
                let crate_arcs = ws.crates();
                let mut known_set = HashSet::new();
                for c_arc in crate_arcs {
                    let locked = c_arc.lock().await;
                    known_set.insert(locked.name().to_string());
                }

                let final_filter = Arc::new(move |nm: &str| {
                    if !show_3p && !known_set.contains(nm) {
                        return false;
                    }
                    if !excl_substr.is_empty() && nm.contains(&excl_substr) {
                        return false;
                    }
                    true
                }) as Arc<dyn Fn(&str)->bool + Send + Sync>;

                let mut config_builder = TopologicalSortConfigBuilder::default();
                config_builder
                    .layering_enabled(layered_flag)
                    .reverse_order(reverse_flag)
                    .remove_unwanted_from_graph(rm_unwanted)
                    .filter_fn(Some(final_filter));
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

    // Helper to ensure we have a workspace path
    fn get_workspace_path(&self, context: &str) -> Result<PathBuf, WorkspaceError> {
        if let Some(ref p) = self.workspace_path {
            Ok(p.clone())
        } else {
            Ok(PathBuf::from("."))
        }
    }
}
