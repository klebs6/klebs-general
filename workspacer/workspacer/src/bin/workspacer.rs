// ---------------- [ File: workspacer/src/bin/workspacer.rs ]
use workspacer_3p::*;
use workspacer_cli::*;
use workspacer_export::*;
use workspacer::*;

/// A top-level wrapper that holds global flags plus the actual `WsCli` command.
/// This way we can parse `--trace` once globally, and decide whether to configure or silence tracing.
#[derive(Debug, StructOpt)]
#[structopt(name = "ws", about = "Workspacer CLI - manage workspaces and crates")]
pub struct WsCli {

    /// If present, enable detailed tracing logs (otherwise silent or minimal logs).
    #[structopt(long = "trace")]
    trace: bool,

    /// The actual subcommand (Add, Analyze, Show, etc.)
    #[structopt(subcommand)]
    cmd: WsCliSubcommand,
}

impl WsCli {

    pub async fn run(self) -> Result<(), WorkspaceError> {

        // 1) If user asked for --trace, configure full tracing. If not, fallback to minimal or silent logs.
        if self.trace {
            configure_tracing();  // your existing tracing setup
        }

        Ok(self.cmd.run().await?)
    }
}

/// Top-level CLI for the `ws` command.
#[derive(Debug, StructOpt)]
#[structopt(name = "ws", about = "Workspacer CLI - manage workspaces and crates")]
pub enum WsCliSubcommand {
    Add               { #[structopt(subcommand)] subcommand: AddSubcommand,               } ,
    Analyze           { #[structopt(subcommand)] subcommand: AnalyzeSubcommand,           } ,
    Bump              { #[structopt(subcommand)] subcommand: BumpSubcommand,              } ,
    CheckPublishReady { #[structopt(subcommand)] subcommand: CheckPublishReadySubcommand, } ,
    Cleanup           { #[structopt(subcommand)] subcommand: CleanupSubcommand,           } ,
    Coverage          { #[structopt(subcommand)] subcommand: CoverageSubcommand,          } ,
    DetectCycles      { #[structopt(subcommand)] subcommand: DetectCyclesSubcommand,      } ,
    Document          { #[structopt(subcommand)] subcommand: DocumentSubcommand,          } ,
    Format            { #[structopt(subcommand)] subcommand: FormatSubcommand,            } ,
    Get               { #[structopt(subcommand)] subcommand: GetSubcommand,               } ,
    Git               { #[structopt(subcommand)] subcommand: GitSubcommand,               } ,
    Info              { #[structopt(subcommand)] subcommand: InfoSubcommand,              } ,
    Lint              { #[structopt(subcommand)] subcommand: LintSubcommand,              } ,
    Meta              { #[structopt(subcommand)] subcommand: MetaSubcommand,              } ,
    Name              { #[structopt(subcommand)] subcommand: NameSubcommand,              } ,
    Organize          { #[structopt(subcommand)] subcommand: OrganizeSubcommand,          } ,
    Pin               { #[structopt(subcommand)] subcommand: PinSubcommand,               } ,
    Publish           { #[structopt(subcommand)] subcommand: PublishSubcommand,           } ,
    Register          { #[structopt(subcommand)] subcommand: RegisterSubcommand,          } ,
    Upgrade           { #[structopt(subcommand)] subcommand: UpgradeSubcommand,           } ,
    Validate          { #[structopt(subcommand)] subcommand: ValidateSubcommand,          } ,
    Watch             { #[structopt(subcommand)] subcommand: WatchSubcommand,             } ,
    Prune             { #[structopt(subcommand)] subcommand: PruneSubcommand,             } ,
    Show              { #[structopt(subcommand)] subcommand: ShowSubcommand,              } ,

    Topo(TopoSubcommand),
    Write(ReadmeWriterCli),
    Tree(TreeSubcommand),
}

impl WsCliSubcommand {

    pub async fn run(self) -> Result<(), WorkspaceError> {
        match self {
            WsCliSubcommand::Add               { subcommand } => { subcommand.run().await },
            WsCliSubcommand::Analyze           { subcommand } => { subcommand.run().await },
            WsCliSubcommand::Bump              { subcommand } => { subcommand.run().await },
            WsCliSubcommand::CheckPublishReady { subcommand } => { subcommand.run().await },
            WsCliSubcommand::Cleanup           { subcommand } => { subcommand.run().await },
            WsCliSubcommand::Coverage          { subcommand } => { subcommand.run().await },
            WsCliSubcommand::DetectCycles      { subcommand } => { subcommand.run().await },
            WsCliSubcommand::Document          { subcommand } => { subcommand.run().await },
            WsCliSubcommand::Format            { subcommand } => { subcommand.run().await },
            WsCliSubcommand::Get               { subcommand } => { subcommand.run().await },
            WsCliSubcommand::Git               { subcommand } => { subcommand.run().await },
            WsCliSubcommand::Info              { subcommand } => { subcommand.run().await },
            WsCliSubcommand::Lint              { subcommand } => { subcommand.run().await },
            WsCliSubcommand::Meta              { subcommand } => { subcommand.run().await },
            WsCliSubcommand::Name              { subcommand } => { subcommand.run().await },
            WsCliSubcommand::Organize          { subcommand } => { subcommand.run().await },
            WsCliSubcommand::Pin               { subcommand } => { subcommand.run().await },
            WsCliSubcommand::Publish           { subcommand } => { subcommand.run().await },
            WsCliSubcommand::Register          { subcommand } => { subcommand.run().await },
            WsCliSubcommand::Upgrade           { subcommand } => { subcommand.run().await },
            WsCliSubcommand::Validate          { subcommand } => { subcommand.run().await },
            WsCliSubcommand::Watch             { subcommand } => { subcommand.run().await },
            WsCliSubcommand::Prune             { subcommand } => { subcommand.run().await },
            WsCliSubcommand::Show              { subcommand } => { subcommand.run().await },

            WsCliSubcommand::Topo(cmd) => { cmd.run().await },
            WsCliSubcommand::Write(cmd) => { cmd.run().await },
            WsCliSubcommand::Tree(cmd)  => { cmd.run().await },
        }
    }
}

#[tokio::main]
async fn main() -> Result<(),WorkspaceError> {

    let cli = WsCli::from_args();

    cli.run().await?;

    Ok(())
}

#[cfg(test)]
mod test_show_subcommand {
    use super::*;
    
    #[traced_test]
    fn test_parse_show_crate_flags() {
        trace!("Testing parse of `ws show crate` with flags");
        // We parse at the top-level with WsCli, so the arguments begin at "ws show crate"
        let args = vec![
            "ws", "show", "crate",
            "--skip-git-check",
            "--include-fn-bodies",
            "--include-private",
            "--path", "my-crate"
        ];
        let cli = WsCli::from_iter_safe(args).expect("Should parse WsCli with subcommand=Show::Crate");
        match cli {
            WsCli::Show { subcommand: ShowSubcommand::Crate(flags) } => {
                assert!(flags.skip_git_check());
                assert!(flags.include_fn_bodies());
                assert!(flags.include_private());
                assert_eq!(flags.path().as_ref().unwrap().to_string_lossy(), "my-crate");
            }
            other => {
                panic!("Expected WsCli::Show {{ subcommand: ShowSubcommand::Crate(...) }}, got {:?}", other);
            }
        }
    }

    #[traced_test]
    fn test_parse_show_crate_and_internal_deps() {
        trace!("Testing parse of `ws show crate-and-internal-deps` with flags");
        let args = vec![
            "ws", "show", "crate-and-internal-deps",
            "--just-structs",
            "--path", "my-other-crate"
        ];
        let cli = WsCli::from_iter_safe(args).expect("Should parse WsCli with subcommand=Show::CrateAndInternalDeps");
        match cli {
            WsCli::Show { subcommand: ShowSubcommand::CrateAndInternalDeps(flags) } => {
                assert!(flags.just_structs());
                assert_eq!(flags.path().as_ref().unwrap().to_string_lossy(), "my-other-crate");
            }
            other => {
                panic!("Expected WsCli::Show {{ subcommand: ShowSubcommand::CrateAndInternalDeps(...) }}, got {:?}", other);
            }
        }
    }

    #[traced_test]
    fn test_parse_show_workspace() {
        trace!("Testing parse of `ws show workspace` with no extra flags");
        let args = vec!["ws", "show", "workspace"];
        let cli = WsCli::from_iter_safe(args).expect("Should parse WsCli with subcommand=Show::Workspace");
        match cli {
            WsCli::Show { subcommand: ShowSubcommand::Workspace(flags) } => {
                assert!(!flags.skip_git_check());
                assert!(flags.path().is_none());
            }
            other => {
                panic!("Expected WsCli::Show {{ subcommand: ShowSubcommand::Workspace(...) }}, got {:?}", other);
            }
        }
    }
}
