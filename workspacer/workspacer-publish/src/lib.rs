// ---------------- [ File: workspacer-publish/src/lib.rs ]
#[macro_use] mod imports; use imports::*;

x!{publish_public_crates_in_topological_order}
x!{try_publish_crate}

// [File: workspacer-publish/src/tests_integration_with_mock_cratesio.rs]
#[cfg(test)]
mod test_integration_with_mock_cratesio {
    use super::*;
    use std::time::Duration;
    use std::thread;
    use rocket::figment::Figment;
    use rocket::figment::providers::{Format, Toml};
    use rocket::Config;
    use portpicker::pick_unused_port;
    use std::env;
    use tracing::{trace, debug, info, warn, error};

    // We'll import the mock crates.io server pieces here, but
    // only in test mode so as not to interfere with normal runs:
    #[allow(unused_imports)]
    #[cfg(test)]
    use workspacer_cratesio_mock::{
        AppStateBuilder, MockCratesDb, publish_new, not_found
    };

    // We also bring in the standard test dependencies from rocket:
    use rocket::tokio::task::JoinHandle;

    /// This test module demonstrates how to spin up the
    /// `workspacer-cratesio-mock` server on a random port and
    /// then point our publish logic at it (so we never hit
    /// the real crates.io during automated tests).
    ///
    /// We do this in such a way that normal usage of `cargo publish`
    /// outside of tests is *not* affected. That is, the environment
    /// variables and any special registry/index config are only
    /// set within this test.
    #[traced_test]
    fn test_publish_against_mock_cratesio() {
        // We'll run async code inside a blocking test by creating
        // our own tokio runtime:
        let rt = tokio::runtime::Runtime::new().unwrap();

        // Execute the async test logic:
        rt.block_on(async {
            trace!("Starting test_publish_against_mock_cratesio.");

            // 1) Pick an unused port for the mock server:
            let port = pick_unused_port().expect("No free ports available");
            info!("Picked unused port: {}", port);

            // 2) Construct the mock server (Rocket) with that port:
            let rocket_config = Config {
                port,
                address: std::net::Ipv4Addr::LOCALHOST.into(),
                ..Config::debug_default()
            };

            let state = AppStateBuilder::default()
                .db(Arc::new(AsyncMutex::new(MockCratesDb::default())))
                .build()
                .unwrap();

            let rocket_instance = rocket::custom(Figment::from(rocket_config))
                .manage(state)
                .mount("/", rocket::routes![publish_new])
                .register("/", rocket::catchers![not_found]);

            // 3) Launch in background:
            debug!("Launching mock crates.io server in background");
            let rocket_handle: JoinHandle<_> = tokio::spawn(async move {
                if let Err(e) = rocket_instance.launch().await {
                    error!("Mock crates.io server failed to launch: {:?}", e);
                }
            });

            // Give Rocket a brief moment to bind the port:
            thread::sleep(Duration::from_millis(250));

            // 4) Override environment variables so our code under test
            //    uses the mock server instead of real crates.io.
            //    For instance, you might override your function that
            //    does `is_crate_version_published_on_crates_io` or
            //    set cargo registry index URLs. We'll show a simple approach:
            let mock_api_url = format!("http://127.0.0.1:{}/api/v1", port);
            info!("Mock server URL: {}", mock_api_url);

            // Suppose your code checks `CARGO_REGISTRIES_CRATES_IO_INDEX` or
            // a custom env var `MOCK_CRATES_IO_URL`. We'll use a fictitious
            // one here for illustration. Adjust to match your real usage.
            let old_env = unsafe { env::var("MOCK_CRATES_IO_URL").ok() };
            unsafe { env::set_var("MOCK_CRATES_IO_URL", &mock_api_url) };

            // 5) Now run the publish logic in your crate that should talk
            //    to the mock instead of real crates.io. For demonstration,
            //    we'll just call a function that might do `try_publish`.
            //    In real usage, you'd call your actual code here:
            let result = run_publish_logic_in_tests().await;
            assert!(result.is_ok(), "Expected no errors during publish logic test");

            // Restore previous environment variable if it existed:
            if let Some(val) = old_env {
                unsafe { env::set_var("MOCK_CRATES_IO_URL", val) };
            } else {
                unsafe { env::remove_var("MOCK_CRATES_IO_URL") };
            }

            // 6) Shut down the rocket server:
            debug!("Shutting down mock crates.io server...");
            rocket_handle.abort();
        });
    }

    /// Example async function that your tests might call,
    /// pretending to run "try_publish" or similar. In a real
    /// scenario, you'd have your crate's production code
    /// that references the environment variable or uses
    /// your "workspacer_check_crates_io" logic, etc.
    async fn run_publish_logic_in_tests() -> Result<(), Box<dyn std::error::Error>> {
        // For illustration: we just log a statement and succeed.
        // In real usage, you'd call your workspace's publish code,
        // which should now talk to the local mock server.
        info!("Pretending to do a publish that hits our mock crates.io...");
        Ok(())
    }
}
