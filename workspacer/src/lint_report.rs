crate::ix!();

#[derive(Debug)]
pub struct LintReport {
    stdout: String,
    stderr: String,
    success: bool,
}

impl LintReport {

    pub fn stdout(&self) -> &str {
        &self.stdout
    }

    pub fn stderr(&self) -> &str {
        &self.stderr
    }

    pub fn success(&self) -> bool {
        self.success
    }
}

impl From<std::process::Output> for LintReport {

    fn from(output: std::process::Output) -> Self {
        Self {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            success: output.status.success(),
        }
    }
}

impl MaybeThrow for LintReport {

    type Error = LintingError;

    fn maybe_throw(&self) -> Result<(),Self::Error> {
        if !self.success() {
            return Err(LintingError::UnknownError {
                stderr: Some(self.stderr.clone()),
                stdout: Some(self.stdout.clone()),
            });
        }

        Ok(())
    }
}
