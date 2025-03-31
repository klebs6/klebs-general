// ---------------- [ File: workspacer-cli/src/show.rs ]
crate::ix!();

/// Options for `ws show` (include or exclude certain items).
///
/// Show consolidated interface with various filtering flags
#[derive(Debug, StructOpt)]
pub struct ShowSubcommand {
    /// Path to the crate (or workspace root) you want to show.
    #[structopt(long = "path", parse(from_os_str))]
    pub path: Option<PathBuf>,

    /// Include private items
    #[structopt(long = "include-private")]
    pub include_private: bool,

    /// Include doc items
    #[structopt(long = "include-docs")]
    pub include_docs: bool,

    /// Include test items
    #[structopt(long = "include-tests")]
    pub include_tests: bool,

    /// Include function bodies
    #[structopt(long = "include-fn-bodies")]
    pub include_fn_bodies: bool,

    /// Include test function bodies
    #[structopt(long = "include-test-bodies")]
    pub include_test_bodies: bool,

    /// Show only test items (skips non-test)
    #[structopt(long = "just-tests")]
    pub just_tests: bool,

    /// Show only free functions (skips impls, structs, etc.)
    #[structopt(long = "just-fns")]
    pub just_fns: bool,

    /// Show only impl blocks
    #[structopt(long = "just-impls")]
    pub just_impls: bool,

    /// Show only traits
    #[structopt(long = "just-traits")]
    pub just_traits: bool,

    /// Show only enums
    #[structopt(long = "just-enums")]
    pub just_enums: bool,

    /// Show only structs
    #[structopt(long = "just-structs")]
    pub just_structs: bool,

    /// Show only type aliases
    #[structopt(long = "just-aliases")]
    pub just_aliases: bool,

    /// Show only ADTs (enums + structs)
    #[structopt(long = "just-adts")]
    pub just_adts: bool,

    /// Show only macros
    #[structopt(long = "just-macros")]
    pub just_macros: bool,

    /// If true, skip the "git clean" check
    #[structopt(long = "skip-git-check")]
    pub skip_git_check: bool,
}

impl ShowSubcommand {

    pub async fn run(&self) -> Result<(), WorkspaceError> {
        // 1) Figure out the path to either a workspace or a single crate
        let path = self.path.clone().unwrap_or_else(|| PathBuf::from("."));

        // 2) Attempt to build a `Workspace` from that path. If it fails with
        //    `WorkspaceError::ActuallyInSingleCrate`, we switch to single-crate logic.
        match Workspace::<PathBuf, CrateHandle>::new(&path).await {
            Ok(workspace) => {
                // This is a real workspace. Optionally do a git-clean check:
                if !self.skip_git_check {
                    workspace.ensure_git_clean().await?;
                }

                // Show consolidated interface for each crate in the workspace (or for the entire workspace).
                // Typically you'd gather all crates, run .consolidate_crate_interface(...) on each,
                // or unify them. For a “show” command, you might iterate:
                for crate_handle in workspace.crates() {
                    let mut guard = crate_handle.lock().await;
                    let cci = guard.consolidate_crate_interface(&self.to_consolidation_opts())
                                   .await
                                   .map_err(|e| WorkspaceError::CrateError(e))?;
                    println!("--- [Crate: {}] ---", guard.name());
                    self.print_filtered(&cci);
                }
            }
            Err(WorkspaceError::ActuallyInSingleCrate { path: _single }) => {
                // single-crate fallback
                let mut single = CrateHandle::new(&path).await.map_err(WorkspaceError::CrateError)?;
                if !self.skip_git_check {
                    single.ensure_git_clean().await.map_err(WorkspaceError::GitError)?;
                }

                let cci = single.consolidate_crate_interface(&self.to_consolidation_opts())
                    .await
                    .map_err(WorkspaceError::CrateError)?;
                self.print_filtered(&cci);
            }
            Err(e) => {
                // other workspace error
                return Err(e);
            }
        }

        Ok(())
    }

    /// Construct a `ConsolidationOptions` from the flags in `ShowSubcommand`.
    fn to_consolidation_opts(&self) -> ConsolidationOptions {
        let mut opts = ConsolidationOptions::new();

        if self.include_docs {
            opts = opts.with_docs();
        }
        if self.include_private {
            opts = opts.with_private_items();
        }
        if self.include_tests {
            opts = opts.with_test_items();
        }
        if self.include_fn_bodies {
            opts = opts.with_fn_bodies();
        }
        if self.include_test_bodies {
            opts = opts.with_fn_bodies_in_tests();
        }
        if self.just_tests {
            // If user wants only test items:
            opts = opts.with_only_test_items();
        }

        // The sub-flags like `just_fns`, `just_structs`, etc., you might implement
        // as “post-filtering” on the final `ConsolidatedCrateInterface` (since your
        // library doesn’t directly have a `ConsolidationOptions` field for “just_fns”).
        // We’ll handle those in `print_filtered` below.
        
        opts.validate();
        opts
    }

    /// Print the resulting consolidated crate interface, applying post-filters
    /// like `just_fns`, `just_impls`, `just_traits`, etc.
    fn print_filtered(&self, cci: &ConsolidatedCrateInterface) {
        // If the user asked for, say, `just_fns`, we print only cci.fns().
        // Or if `just_enums`, we print only cci.enums(). etc.

        if self.just_fns {
            for (i, item) in cci.fns().iter().enumerate() {
                println!("{}", item);
                if i + 1 < cci.fns().len() {
                    println!();
                }
            }
            return;
        }
        if self.just_impls {
            for (i, ib) in cci.impls().iter().enumerate() {
                println!("{}", ib);
                if i + 1 < cci.impls().len() {
                    println!();
                }
            }
            return;
        }
        if self.just_traits {
            for (i, tr) in cci.traits().iter().enumerate() {
                println!("{}", tr);
                if i + 1 < cci.traits().len() {
                    println!();
                }
            }
            return;
        }
        if self.just_enums {
            for (i, en) in cci.enums().iter().enumerate() {
                println!("{}", en);
                if i + 1 < cci.enums().len() {
                    println!();
                }
            }
            return;
        }
        if self.just_structs {
            for (i, st) in cci.structs().iter().enumerate() {
                println!("{}", st);
                if i + 1 < cci.structs().len() {
                    println!();
                }
            }
            return;
        }
        if self.just_aliases {
            for (i, ta) in cci.type_aliases().iter().enumerate() {
                println!("{}", ta);
                if i + 1 < cci.type_aliases().len() {
                    println!();
                }
            }
            return;
        }
        if self.just_adts {
            // ADTs => enums + structs. We print them in whichever order you prefer.
            let mut combined: Vec<String> = Vec::new();
            for e in cci.enums() {
                combined.push(format!("{}", e));
            }
            for s in cci.structs() {
                combined.push(format!("{}", s));
            }
            for (i, out_str) in combined.iter().enumerate() {
                println!("{}", out_str);
                if i + 1 < combined.len() {
                    println!();
                }
            }
            return;
        }
        if self.just_macros {
            for (i, mac) in cci.macros().iter().enumerate() {
                println!("{}", mac);
                if i + 1 < cci.macros().len() {
                    println!();
                }
            }
            return;
        }

        // Otherwise, print the entire consolidated interface as is:
        println!("{}", cci);
    }
}
