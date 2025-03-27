// ---------------- [ File: workspacer/src/bin/workspacer.rs ]
use workspacer_3p::*;
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
    Describe          { #[structopt(subcommand)] subcommand: DescribeSubcommand,          } ,
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
    Write             { #[structopt(subcommand)] subcommand: WriteSubcommand,             } ,
}

#[tokio::main]
async fn main() {
    match WsCli::from_args() {
        WsCli::Add               { subcommand } => { subcommand.run() },
        WsCli::Analyze           { subcommand } => { subcommand.run() },
        WsCli::Bump              { subcommand } => { subcommand.run() },
        WsCli::CheckPublishReady { subcommand } => { subcommand.run() },
        WsCli::Cleanup           { subcommand } => { subcommand.run() },
        WsCli::Coverage          { subcommand } => { subcommand.run() },
        WsCli::Describe          { subcommand } => { subcommand.run() },
        WsCli::DetectCycles      { subcommand } => { subcommand.run() },
        WsCli::Document          { subcommand } => { subcommand.run() },
        WsCli::Format            { subcommand } => { subcommand.run() },
        WsCli::Get               { subcommand } => { subcommand.run() },
        WsCli::Git               { subcommand } => { subcommand.run() },
        WsCli::Info              { subcommand } => { subcommand.run() },
        WsCli::Lint              { subcommand } => { subcommand.run() },
        WsCli::Meta              { subcommand } => { subcommand.run() },
        WsCli::Name              { subcommand } => { subcommand.run() },
        WsCli::Organize          { subcommand } => { subcommand.run() },
        WsCli::Pin               { subcommand } => { subcommand.run() },
        WsCli::Publish           { subcommand } => { subcommand.run() },
        WsCli::Register          { subcommand } => { subcommand.run() },
        WsCli::Show              { subcommand } => { subcommand.run() },
        WsCli::Tree              { subcommand } => { subcommand.run() },
        WsCli::Upgrade           { subcommand } => { subcommand.run() },
        WsCli::Validate          { subcommand } => { subcommand.run() },
        WsCli::Watch             { subcommand } => { subcommand.run() },
        WsCli::Write             { subcommand } => { subcommand.run() },
    }
}
