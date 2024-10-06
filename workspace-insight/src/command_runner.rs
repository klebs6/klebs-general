crate::ix!();

use tokio::process::Command;
use std::io;
use std::process::Output;
use tokio::task::JoinHandle;

pub trait CommandRunner: Send + Sync {
    fn run_command(&self, cmd: Command) -> JoinHandle<Result<Output, io::Error>>;
}

pub struct DefaultCommandRunner;

impl CommandRunner for DefaultCommandRunner {
    fn run_command(&self, cmd: Command) -> JoinHandle<Result<Output, io::Error>> {
        tokio::spawn(async move {
            let mut cmd = cmd;
            cmd.output().await
        })
    }
}
