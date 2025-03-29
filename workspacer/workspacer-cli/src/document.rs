// ---------------- [ File: workspacer-cli/src/document.rs ]
crate::ix!();

/// Document the workspace or crate (like cargo doc)
#[derive(Debug, StructOpt)]
pub enum DocumentSubcommand {
    Crate {
        #[structopt(long = "crate")]
        crate_name: PathBuf,
    },
    Workspace {
        #[structopt(long = "path")]
        path: PathBuf,
    },
}

impl DocumentSubcommand {
    pub async fn run(&self) -> Result<(),WorkspaceError> {
        todo!("We have not implemented this functionality yet. 
            The goal of this command is to use a language model to scrub the whole crate.
            The language model will ensure that everything is properly documented. 
            It is possible that we will implement this once it has become critical path. 
            In the meantime -- to help implement this functionality, please visit https://github.com/klebs6/klebs-general to submit a PR");
    }
}
