// ---------------- [ File: src/command_runner.rs ]
crate::ix!();

pub trait CommandRunner: Send + Sync {

    fn run_command(&self, cmd: tokio::process::Command) 
        -> tokio::task::JoinHandle<Result<std::process::Output, io::Error>>;
}

pub struct DefaultCommandRunner;

impl CommandRunner for DefaultCommandRunner {

    fn run_command(&self, cmd: tokio::process::Command) 
        -> tokio::task::JoinHandle<Result<std::process::Output, io::Error>> 
    {
        tokio::spawn(async move {
            let mut cmd = cmd;
            cmd.output().await
        })
    }
}
