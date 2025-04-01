// ---------------- [ File: workspacer/src/bin/workspacer.rs ]
use workspacer_3p::*;
use workspacer_cli::*;
use workspacer_export::*;
use workspacer::*;

/// Top-level CLI for the `ws` command.
#[derive(Debug, StructOpt)]
#[structopt(name = "ws", about = "Workspacer CLI - manage workspaces and crates")]
pub enum WsCli {
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
    Show              { #[structopt(subcommand)] subcommand: ShowSubcommand,              } ,
    Tree              { #[structopt(subcommand)] subcommand: TreeSubcommand,              } ,
    Upgrade           { #[structopt(subcommand)] subcommand: UpgradeSubcommand,           } ,
    Validate          { #[structopt(subcommand)] subcommand: ValidateSubcommand,          } ,
    Watch             { #[structopt(subcommand)] subcommand: WatchSubcommand,             } ,
    Write             { #[structopt(subcommand)] subcommand: ReadmeWriterCli,             } ,
    Prune             { #[structopt(subcommand)] subcommand: PruneSubcommand,             } ,
}

#[tokio::main]
async fn main() -> Result<(),WorkspaceError> {
    configure_tracing();
    match WsCli::from_args() {
        WsCli::Add               { subcommand } => { subcommand.run().await? },
        WsCli::Analyze           { subcommand } => { subcommand.run().await? },
        WsCli::Bump              { subcommand } => { subcommand.run().await? },
        WsCli::CheckPublishReady { subcommand } => { subcommand.run().await? },
        WsCli::Cleanup           { subcommand } => { subcommand.run().await? },
        WsCli::Coverage          { subcommand } => { subcommand.run().await? },
        WsCli::DetectCycles      { subcommand } => { subcommand.run().await? },
        WsCli::Document          { subcommand } => { subcommand.run().await? },
        WsCli::Format            { subcommand } => { subcommand.run().await? },
        WsCli::Get               { subcommand } => { subcommand.run().await? },
        WsCli::Git               { subcommand } => { subcommand.run().await? },
        WsCli::Info              { subcommand } => { subcommand.run().await? },
        WsCli::Lint              { subcommand } => { subcommand.run().await? },
        WsCli::Meta              { subcommand } => { subcommand.run().await? },
        WsCli::Name              { subcommand } => { subcommand.run().await? },
        WsCli::Organize          { subcommand } => { subcommand.run().await? },
        WsCli::Pin               { subcommand } => { subcommand.run().await? },
        WsCli::Publish           { subcommand } => { subcommand.run().await? },
        WsCli::Register          { subcommand } => { subcommand.run().await? },
        WsCli::Show              { subcommand } => { subcommand.run().await? },
        WsCli::Tree              { subcommand } => { subcommand.run().await? },
        WsCli::Upgrade           { subcommand } => { subcommand.run().await? },
        WsCli::Validate          { subcommand } => { subcommand.run().await? },
        WsCli::Watch             { subcommand } => { subcommand.run().await? },
        WsCli::Write             { subcommand } => { subcommand.run().await? },
        WsCli::Prune             { subcommand } => { subcommand.run().await? },
    }

    Ok(())
}
