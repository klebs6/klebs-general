// ---------------- [ File: workspacer-cli/src/get.rs ]
crate::ix!();

#[derive(Debug, StructOpt)]
pub enum GetSubcommand {
    /// Get lock-versions from the Cargo.lock
    LockVersions {
        #[structopt(long = "path")]
        workspace_path: Option<PathBuf>,

        #[structopt(long = "skip-git-check")]
        skip_git_check: bool,
    },

    /// Get toml section with optional crate selection
    Toml {
        #[structopt(long = "section")]
        section: String,

        #[structopt(long = "crate")]
        crate_name: Option<String>,
    },
}

impl GetSubcommand {
    /// Main entrypoint: routes to the chosen variant's logic.
    pub async fn run(&self) -> Result<(), WorkspaceError> {
        match self {
            // 1) If user typed `ws get lock-versions ...`
            GetSubcommand::LockVersions {
                workspace_path,
                skip_git_check,
            } => {
                get_lock_versions_flow(
                    workspace_path.clone(),
                    *skip_git_check
                ).await
            },

            // 2) The "ws get toml" subcommand
            GetSubcommand::Toml { section, crate_name } => {
                get_toml_section_flow(
                    section.clone(),
                    crate_name.clone(),
                ).await
            }
        }
    }
}
