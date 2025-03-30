// ---------------- [ File: workspacer-cli/src/git.rs ]
crate::ix!();

/// Subcommands for `ws git`
#[derive(Debug, StructOpt)]
pub enum GitSubcommand {
    /// Perform a git commit
    Commit,
}

impl GitSubcommand {
    pub async fn run(&self) -> Result<(),WorkspaceError> {
        todo!(
            "This functionality is not implemented yet. 
            This command will scan the changes to be committed and partition them sets corresponding to their crate of origin.
            Treating these sets in crate topology order, this command will use a language model to craft a maximally useful commit message for the set of changes, aware of the interface to the crate (via workspacer-consolidate).
            The command will then apply the ai generated commits in playback/dependency order such that we can meaningfully scroll through our commit history and see what is going on at each step.
            To help implement this functionality, please visit https://github.com/klebs6/klebs-general to submit a PR"
        );
    }
}
