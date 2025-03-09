// ---------------- [ File: src/stdout_backup.rs ]
crate::ix!();

#[derive(Debug, Builder)]
#[builder(setter(into))]
pub struct StdoutBackup {
    stdout_fd: i32,
    backup_fd: i32,
}

impl StdoutBackup {
    pub fn new() -> io::Result<Self> {
        let stdout_fd = io::stdout().as_raw_fd();
        eprintln!(
            "[StdoutBackup::new] duplicating current stdout_fd={} with libc::dup()",
            stdout_fd
        );
        let backup_fd = unsafe { libc::dup(stdout_fd) };
        if backup_fd == -1 {
            let err = io::Error::last_os_error();
            eprintln!("[StdoutBackup::new] ERROR: dup() failed: {:?}", err);
            Err(err)
        } else {
            eprintln!(
                "[StdoutBackup::new] success: new backup_fd={} to represent the old stdout",
                backup_fd
            );
            Ok(Self { stdout_fd, backup_fd })
        }
    }

    pub fn restore(&self) -> io::Result<()> {
        eprintln!(
            "[StdoutBackup::restore] about to call libc::dup2(backup_fd={}, stdout_fd={})",
            self.backup_fd, self.stdout_fd
        );
        let rc = unsafe { libc::dup2(self.backup_fd, self.stdout_fd) };
        if rc == -1 {
            let err = io::Error::last_os_error();
            eprintln!(
                "[StdoutBackup::restore] ERROR: dup2({}, {}) failed: {:?}",
                self.backup_fd, self.stdout_fd, err
            );
            Err(err)
        } else {
            eprintln!(
                "[StdoutBackup::restore] success: stdout now restored to fd={}",
                self.backup_fd
            );
            Ok(())
        }
    }
}

impl Drop for StdoutBackup {
    fn drop(&mut self) {
        eprintln!(
            "[StdoutBackup::drop] about to dup2(backup_fd={}, stdout_fd={}) (ignoring errors)",
            self.backup_fd, self.stdout_fd
        );
        let rc_dup2 = unsafe { libc::dup2(self.backup_fd, self.stdout_fd) };
        if rc_dup2 == -1 {
            let err = io::Error::last_os_error();
            eprintln!("[StdoutBackup::drop] WARNING: dup2 failed: {:?}", err);
        } else {
            eprintln!(
                "[StdoutBackup::drop] dup2 succeeded; restored stdout to backup_fd={}",
                self.backup_fd
            );
        }

        eprintln!(
            "[StdoutBackup::drop] now closing backup_fd={} (ignoring errors)",
            self.backup_fd
        );
        let rc_close = unsafe { libc::close(self.backup_fd) };
        if rc_close == -1 {
            let err = io::Error::last_os_error();
            eprintln!(
                "[StdoutBackup::drop] WARNING: close(backup_fd={}) failed: {:?}",
                self.backup_fd, err
            );
        } else {
            eprintln!(
                "[StdoutBackup::drop] closed backup_fd={} successfully",
                self.backup_fd
            );
        }
    }
}
