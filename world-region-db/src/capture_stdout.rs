// ---------------- [ File: src/capture_stdout.rs ]
crate::ix!();

/// Captures everything printed to stdout within a closure `f()`, returning
/// it as a `String`. We lock a global mutex so no other test thread can 
/// manipulate or write to stdout concurrently.
///
/// **NOTE**: If your entire test suite is run with `--test-threads=1` or
/// each test using `#[serial]`, you can omit this global lock. But the lock
/// is a safe extra layer in case other code touches stdout in parallel.
///
/// # Safety
/// - We do an `unsafe` call to `libc::dup2(...)`. This is safe only if no other
///   threads are using stdout concurrently (which is why we lock).
pub fn capture_stdout<F: FnOnce()>(f: F) -> io::Result<String> {

    /// Initialize the global lock on first use.
    fn capture_stdout_global_lock() -> &'static Mutex<()> {
        /// A global lock ensuring that only one `capture_stdout` call runs at a time.
        /// We use `OnceLock` so it’s lazily initialized and a `static Mutex`.
        static STDOUT_CAPTURE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

        eprintln!("locking global lock");
        STDOUT_CAPTURE_LOCK.get_or_init(|| Mutex::new(()))
    }

    // Acquire the global lock so no other thread manipulates stdout:
    //
    // Instead of .expect(...) which fails on poison, we recover from poison:
    let _lock_guard = match capture_stdout_global_lock().lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            warn!("capture_stdout: global lock was poisoned; continuing anyway.");
            poisoned.into_inner() // recover guard from the poisoned lock
        }
    };

    eprintln!("Backing up original stdout");
    let backup = StdoutBackup::new()?;

    eprintln!("Creating an os_pipe so we can read what’s written");
    let (mut reader, writer) = os_pipe::pipe()?;

    eprintln!("Redirecting stdout to the pipe's writer");
    let stdout_fd = io::stdout().as_raw_fd();
    if unsafe { libc::dup2(writer.as_raw_fd(), stdout_fd) } == -1 {
        error!("dup2 call returned -1");
        return Err(io::Error::last_os_error());
    }

    eprintln!("Running the user closure");
    f();

    eprintln!("Flushing stdout so that everytihgn is definitely in the pipe");
    std::io::stdout().flush()?;

    eprintln!("Restoring stdout");
    backup.restore()?;

    eprintln!("Dropping the writer so the read end sees EOF once data is read");
    drop(writer);

    eprintln!("Reading the entire pipe");
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;

    eprintln!("Converting from possibly non-utf8 to a string using .from_utf8_lossy()");
    Ok(String::from_utf8_lossy(&buffer).to_string())
}

#[cfg(test)]
mod capture_stdout_tests {
    // see tests/manual_harness.rs for these
}
