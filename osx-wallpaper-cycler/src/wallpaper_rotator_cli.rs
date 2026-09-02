// ---------------- [ File: osx-wallpaper-cycler/src/wallpaper_rotator_cli.rs ]
crate::ix!();

#[derive(Debug, structopt::StructOpt)]
#[structopt(name = "dropbox-wallpaper-rotator")]
#[structopt(about = "Rotate macOS wallpapers from Dropbox using async HTTP + tokio")]
pub struct WallpaperRotatorCli {
    #[structopt(subcommand)]
    command: WallpaperRotatorCliCommand,
}

impl WallpaperRotatorCli {
    pub fn command(&self) -> &WallpaperRotatorCliCommand {
        &self.command
    }
}

#[derive(Debug, structopt::StructOpt)]
pub enum WallpaperRotatorCliCommand {
    #[structopt(name = "daemon")]
    Daemon {
        #[structopt(long)]
        config: std::path::PathBuf,
    },
    #[structopt(name = "run-once")]
    RunOnce {
        #[structopt(long)]
        config: std::path::PathBuf,
    },
    #[structopt(name = "print-config-template")]
    PrintConfigTemplate,
    #[structopt(name = "print-launch-agent-template")]
    PrintLaunchAgentTemplate {
        #[structopt(long)]
        binary_path: std::path::PathBuf,
        #[structopt(long)]
        config_path: std::path::PathBuf,
    },
}

#[cfg(test)]
mod wallpaper_rotator_cli_parsing_contract_suite {
    use super::*;

    #[traced_test]
    fn cli_parses_daemon_subcommand_with_config_path() {
        let args = <WallpaperRotatorCli as structopt::StructOpt>::from_iter(&[
            "dropbox-wallpaper-rotator",
            "daemon",
            "--config",
            "/tmp/cfg.toml",
        ]);

        match args.command() {
            WallpaperRotatorCliCommand::Daemon { config } => {
                assert_eq!(config.as_path(), std::path::Path::new("/tmp/cfg.toml"));
            }
            _ => panic!("unexpected command"),
        }
    }

    #[traced_test]
    fn cli_parses_run_once_subcommand_with_config_path() {
        let args = <WallpaperRotatorCli as structopt::StructOpt>::from_iter(&[
            "dropbox-wallpaper-rotator",
            "run-once",
            "--config",
            "/tmp/cfg.toml",
        ]);

        match args.command() {
            WallpaperRotatorCliCommand::RunOnce { config } => {
                assert_eq!(config.as_path(), std::path::Path::new("/tmp/cfg.toml"));
            }
            _ => panic!("unexpected command"),
        }
    }

    #[traced_test]
    fn cli_parses_print_templates_subcommands() {
        let args = <WallpaperRotatorCli as structopt::StructOpt>::from_iter(&[
            "dropbox-wallpaper-rotator",
            "print-config-template",
        ]);

        match args.command() {
            WallpaperRotatorCliCommand::PrintConfigTemplate => {}
            _ => panic!("unexpected command"),
        }

        let args = <WallpaperRotatorCli as structopt::StructOpt>::from_iter(&[
            "dropbox-wallpaper-rotator",
            "print-launch-agent-template",
            "--binary-path",
            "/tmp/bin",
            "--config-path",
            "/tmp/cfg.toml",
        ]);

        match args.command() {
            WallpaperRotatorCliCommand::PrintLaunchAgentTemplate {
                binary_path,
                config_path,
            } => {
                assert_eq!(binary_path.as_path(), std::path::Path::new("/tmp/bin"));
                assert_eq!(config_path.as_path(), std::path::Path::new("/tmp/cfg.toml"));
            }
            _ => panic!("unexpected command"),
        }
    }
}
